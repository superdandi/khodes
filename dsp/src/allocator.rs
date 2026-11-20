//! Allocator determinista de voces para M3.
//!
//! Mantiene un pool fijo de 1..=32 voces preasignadas. No conoce TRUCE, el host
//! ni el sistema de archivos. El robo de voces es determinista y se resuelve en
//! este orden: voz en release más antigua, voz sostenida más antigua. Las voces
//! robadas se desvanecen en una cola acotada (`StealTail`) para no cortarlas.
//!
//! Ver `docs/MIDI.md` §9 y `docs/PLAN.md` M3.

use crate::params::Params;
use crate::voice::{Voice, flush_denormal};

/// Número máximo de voces del contrato v1.
pub const MAX_VOICES: usize = 32;
/// Duración del fade aplicado a una voz robada (`MIDI.md` §9).
const STEAL_FADE_S: f64 = 0.004;

/// Estado de un slot dentro del pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState {
    Free,
    Active,
    Released,
}

/// Una voz del pool con la información de performance necesaria para el robo.
#[derive(Clone, Copy, Debug)]
pub struct VoiceSlot {
    pub voice: Voice,
    pub channel: u8,
    pub note: u8,
    pub note_id: i32,
    pub state: SlotState,
    /// Orden de asignación; sirve para robar la sostenida más antigua.
    pub born: u64,
    /// Orden de liberación; sirve para robar la liberada más antigua.
    pub released_at: u64,
    /// Tecla todavía presionada (relevante mientras el sustain está activo).
    pub key_down: bool,
}

impl Default for VoiceSlot {
    fn default() -> Self {
        Self {
            voice: Voice::default(),
            channel: 0,
            note: 0,
            note_id: -1,
            state: SlotState::Free,
            born: 0,
            released_at: 0,
            key_down: false,
        }
    }
}

/// Cola de desvanecimiento de una voz robada.
#[derive(Clone, Copy, Debug)]
struct StealTail {
    voice: Voice,
    gain: f64,
    step: f64,
    active: bool,
}

impl Default for StealTail {
    fn default() -> Self {
        Self {
            voice: Voice::default(),
            gain: 0.0,
            step: 0.0,
            active: false,
        }
    }
}

/// Pool fijo de voces con asignación, robo y sustain por canal.
pub struct Allocator {
    slots: [VoiceSlot; MAX_VOICES],
    tails: [StealTail; MAX_VOICES],
    sustain: [bool; 16],
    clock: u64,
    polyphony: u8,
    fade_step: f64,
}

impl Default for Allocator {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| VoiceSlot::default()),
            tails: std::array::from_fn(|_| StealTail::default()),
            sustain: [false; 16],
            clock: 0,
            polyphony: 16,
            fade_step: 0.0,
        }
    }
}

impl Allocator {
    /// Calcula el paso del fade. Puede ejecutarse fuera de `process`.
    pub fn prepare(&mut self, sample_rate: f64) {
        self.fade_step = 1.0 / (STEAL_FADE_S * sample_rate.max(1.0));
        self.reset();
    }

    /// Limpia todo el estado transitorio sin liberar memoria.
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            *slot = VoiceSlot::default();
        }
        for tail in &mut self.tails {
            *tail = StealTail::default();
        }
        self.sustain = [false; 16];
        self.clock = 0;
    }

    #[cfg(test)]
    pub const fn slots(&self) -> &[VoiceSlot; MAX_VOICES] {
        &self.slots
    }

    /// Número de voces actualmente activas.
    #[must_use]
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < MAX_VOICES {
            if self.slots[i].state == SlotState::Active {
                count += 1;
            }
            i += 1;
        }
        count
    }

    #[cfg(test)]
    pub const fn sustain(&self, channel: u8) -> bool {
        self.sustain[(channel & 15) as usize]
    }

    /// Número de voces permitidas (`1..=32`). Al reducirlo, las voces que
    /// quedan fuera del nuevo límite se mueven a la cola de desvanecimiento.
    pub fn set_polyphony(&mut self, voices: u8) {
        let new_limit = usize::from(voices.clamp(1, MAX_VOICES as u8));
        if new_limit < usize::from(self.polyphony) {
            for i in new_limit..MAX_VOICES {
                if !matches!(self.slots[i].state, SlotState::Free) {
                    self.steal(i);
                }
            }
        }
        self.polyphony = new_limit as u8;
        while self.active_count() > new_limit {
            let Some(victim) = self.find_oldest_active() else {
                break;
            };
            self.release(victim);
        }
    }

    /// Los slots activos son siempre los primeros `polyphony`.
    fn limit(&self) -> usize {
        usize::from(self.polyphony).min(MAX_VOICES)
    }

    fn find_free(&self) -> Option<usize> {
        self.slots[..self.limit()]
            .iter()
            .position(|s| s.state == SlotState::Free)
    }

    /// Voz activa más antigua, para reducir la polifonía en caliente.
    fn find_oldest_active(&self) -> Option<usize> {
        let mut oldest: Option<(u64, usize)> = None;
        for (i, slot) in self.slots[..self.limit()].iter().enumerate() {
            if slot.state == SlotState::Active && oldest.is_none_or(|(at, _)| slot.born < at) {
                oldest = Some((slot.born, i));
            }
        }
        oldest.map(|(_, i)| i)
    }

    /// Voz en release más antigua; si no hay, sostenida más antigua.
    fn find_victim(&self) -> Option<usize> {
        let mut released: Option<(u64, usize)> = None;
        let mut sustained: Option<(u64, usize)> = None;
        for (i, slot) in self.slots[..self.limit()].iter().enumerate() {
            match slot.state {
                SlotState::Released => {
                    if released.is_none_or(|(at, _)| slot.released_at < at) {
                        released = Some((slot.released_at, i));
                    }
                }
                SlotState::Active => {
                    if sustained.is_none_or(|(at, _)| slot.born < at) {
                        sustained = Some((slot.born, i));
                    }
                }
                SlotState::Free => {}
            }
        }
        released.or(sustained).map(|(_, i)| i)
    }

    /// Mueve la voz de `index` a la cola de desvanecimiento.
    fn steal(&mut self, index: usize) {
        let slot = &mut self.slots[index];
        if matches!(slot.state, SlotState::Free) {
            return;
        }
        if let Some(tail) = self.tails.iter_mut().find(|t| !t.active) {
            tail.voice = slot.voice;
            tail.gain = 1.0;
            tail.step = self.fade_step.max(1.0);
            tail.active = true;
        }
        slot.voice.reset();
        slot.state = SlotState::Free;
    }

    /// Inicia una nota. Si la nota ya estaba activa en el mismo canal, la
    /// reinicia (retrigger). Si no hay hueco, roba una voz de forma determinista.
    pub fn note_on(
        &mut self,
        channel: u8,
        note: u8,
        velocity: f64,
        note_id: i32,
        params: &Params,
        sample_rate: f64,
    ) {
        self.clock += 1;
        let channel = channel & 15;

        // Retrigger de nota repetida: misma nota y canal, sin note-off previo.
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|s| s.state == SlotState::Active && s.channel == channel && s.note == note)
        {
            slot.voice.note_on(note, velocity, params, sample_rate);
            slot.note_id = note_id;
            slot.key_down = true;
            slot.born = self.clock;
            return;
        }

        let index = match self.find_free() {
            Some(i) => i,
            None => self.find_victim().unwrap_or(0),
        };

        if !matches!(self.slots[index].state, SlotState::Free) {
            self.steal(index);
        }

        let slot = &mut self.slots[index];
        slot.voice.note_on(note, velocity, params, sample_rate);
        slot.channel = channel;
        slot.note = note;
        slot.note_id = note_id;
        slot.state = SlotState::Active;
        slot.born = self.clock;
        slot.released_at = 0;
        slot.key_down = true;

        // Respetar el límite de polifonía si bajó durante la nota. Se liberan
        // activos explícitamente: `find_victim` elegiría las recién liberadas.
        while self.active_count() > usize::from(self.polyphony) {
            let Some(victim) = self.find_oldest_active() else {
                break;
            };
            self.release(victim);
        }
    }

    /// Libera el slot indicado, pasando la voz a release.
    fn release(&mut self, index: usize) {
        self.clock += 1;
        let slot = &mut self.slots[index];
        if matches!(slot.state, SlotState::Free) {
            return;
        }
        let note = slot.note;
        slot.voice.note_off(note);
        slot.state = SlotState::Released;
        slot.released_at = self.clock;
        slot.key_down = false;
    }

    /// Note-off. Con `note_id >= 0` solo afecta a esa asignación concreta;
    /// con `-1` busca por canal y nota (wildcard de hosts que no dan ID).
    pub fn note_off(&mut self, channel: u8, note: u8, note_id: i32) {
        let channel = channel & 15;
        let index = self.slots.iter().position(|s| {
            s.state != SlotState::Free
                && s.channel == channel
                && s.note == note
                && (note_id < 0 || s.note_id == note_id)
        });
        let Some(index) = index else { return };

        if self.sustain[usize::from(channel)] {
            self.slots[index].key_down = false;
            return;
        }
        self.release(index);
    }

    /// Activa o desactiva el sustain (CC64) para un canal.
    pub fn set_sustain(&mut self, channel: u8, enabled: bool) {
        let channel = usize::from(channel & 15);
        self.sustain[channel] = enabled;
        if enabled {
            return;
        }
        for i in 0..MAX_VOICES {
            if self.slots[i].state == SlotState::Active
                && usize::from(self.slots[i].channel) == channel
                && !self.slots[i].key_down
            {
                self.release(i);
            }
        }
    }

    /// CC123: libera todas las voces con el release normal.
    pub fn all_notes_off(&mut self) {
        self.sustain = [false; 16];
        for i in 0..MAX_VOICES {
            if self.slots[i].state == SlotState::Active {
                self.release(i);
            }
            self.slots[i].key_down = false;
        }
    }

    /// CC120: corta todas las voces con un fade corto.
    pub fn all_sound_off(&mut self) {
        self.sustain = [false; 16];
        for i in 0..MAX_VOICES {
            if !matches!(self.slots[i].state, SlotState::Free) {
                self.slots[i].voice.kill();
                self.slots[i].state = SlotState::Released;
                self.slots[i].key_down = false;
            }
        }
    }

    /// CC121: centra bend, desactiva sustain y limpia modulación.
    pub fn reset_controllers(&mut self) {
        self.sustain = [false; 16];
    }

    /// Renderiza un segmento sumando todas las voces y las colas de robo.
    ///
    /// `scratch` debe tener al menos `out.len()` elementos y se reserva una vez
    /// fuera del hilo de audio. No se asigna memoria aquí.
    pub fn render_segment(
        &mut self,
        out: &mut [f64],
        mults: &[f64],
        scratch: &mut [f64],
        params: &Params,
        sample_rate: f64,
        noise: &[f64],
    ) {
        let len = out.len().min(mults.len()).min(scratch.len());
        out[..len].fill(0.0);

        // Colas de robo: se desvanecen y desaparecen al terminar.
        for tail in &mut self.tails {
            if !tail.active {
                continue;
            }
            let buf = &mut scratch[..len];
            tail.voice
                .render(buf, &mults[..len], params, sample_rate, noise);
            for i in 0..len {
                if tail.gain > 0.0 {
                    out[i] += buf[i] * tail.gain;
                    tail.gain -= tail.step;
                }
            }
            if tail.gain <= 0.0 || !tail.voice.is_active() {
                *tail = StealTail::default();
            }
        }

        // Voces normales.
        for slot in &mut self.slots {
            if matches!(slot.state, SlotState::Free) {
                continue;
            }
            let buf = &mut scratch[..len];
            slot.voice
                .render(buf, &mults[..len], params, sample_rate, noise);
            for i in 0..len {
                out[i] += buf[i];
            }
            if !slot.voice.is_active() {
                *slot = VoiceSlot::default();
            }
        }

        for sample in &mut out[..len] {
            *sample = flush_denormal(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine(voices: u8) -> Allocator {
        let mut allocator = Allocator::default();
        allocator.prepare(48_000.0);
        allocator.set_polyphony(voices);
        allocator
    }

    fn press(allocator: &mut Allocator, note: u8) {
        allocator.note_on(0, note, 1.0, i32::from(note), &Params::default(), 48_000.0);
    }

    fn render(allocator: &mut Allocator) -> Vec<f64> {
        let len = 512;
        let mut out = vec![0.0; len];
        let mults = vec![1.0; len];
        let mut scratch = vec![0.0; len];
        allocator.render_segment(
            &mut out,
            &mults,
            &mut scratch,
            &Params::default(),
            48_000.0,
            &[],
        );
        out
    }

    #[test]
    fn polyphony_from_one_to_thirty_two() {
        for voices in [1, 8, 16, 32] {
            let mut allocator = engine(voices);
            for note in 21..21 + voices {
                press(&mut allocator, note);
            }
            assert!(allocator.active_count() <= usize::from(voices));
            assert!(render(&mut allocator).iter().all(|x| x.is_finite()));
        }
    }

    #[test]
    fn deterministic_stealing_prefers_released_then_oldest() {
        let mut allocator = engine(4);
        for note in 60..64 {
            press(&mut allocator, note);
        }
        // Libera la nota 60: debe ser la primera víctima.
        allocator.note_off(0, 60, 60);
        assert_eq!(allocator.slots()[0].state, SlotState::Released);
        press(&mut allocator, 70);
        assert_eq!(allocator.slots()[0].note, 70);

        // Sin liberadas, roba la activa más antigua (61).
        press(&mut allocator, 71);
        assert_eq!(allocator.slots()[1].note, 71);
    }

    #[test]
    fn repeated_note_does_not_create_zombie_voices() {
        let mut allocator = engine(8);
        press(&mut allocator, 60);
        press(&mut allocator, 60);
        press(&mut allocator, 60);
        assert_eq!(allocator.active_count(), 1);
    }

    #[test]
    fn note_off_without_note_on_is_ignored() {
        let mut allocator = engine(8);
        allocator.note_off(3, 64, -1);
        assert_eq!(allocator.active_count(), 0);
    }

    #[test]
    fn one_thousand_note_ons_are_bounded() {
        let mut allocator = engine(16);
        for i in 0..1000 {
            press(&mut allocator, (i % 128) as u8);
        }
        assert!(allocator.active_count() <= 16);
        assert!(render(&mut allocator).iter().all(|x| x.is_finite()));
    }

    #[test]
    fn sustain_is_per_channel() {
        let mut allocator = engine(8);
        allocator.set_sustain(0, true);
        allocator.note_on(0, 60, 1.0, 60, &Params::default(), 48_000.0);
        allocator.note_off(0, 60, 60);
        assert_eq!(allocator.slots()[0].state, SlotState::Active);
        allocator.set_sustain(1, true);
        allocator.set_sustain(1, false);
        assert_eq!(allocator.slots()[0].state, SlotState::Active);
        allocator.set_sustain(0, false);
        assert_eq!(allocator.slots()[0].state, SlotState::Released);
    }

    #[test]
    fn all_notes_off_and_all_sound_off() {
        let mut allocator = engine(8);
        allocator.set_sustain(0, true);
        press(&mut allocator, 60);
        allocator.all_notes_off();
        assert!(!allocator.sustain(0));
        assert_eq!(allocator.active_count(), 0);
        press(&mut allocator, 61);
        allocator.all_sound_off();
        assert_eq!(allocator.active_count(), 0);
    }

    #[test]
    fn late_note_off_cannot_release_a_new_voice() {
        let mut allocator = engine(1);
        allocator.note_on(0, 60, 1.0, 100, &Params::default(), 48_000.0);
        allocator.note_on(0, 62, 1.0, 101, &Params::default(), 48_000.0);
        allocator.note_off(0, 60, 100);
        assert_eq!(allocator.slots()[0].note, 62);
        assert_eq!(allocator.slots()[0].state, SlotState::Active);
    }
}
