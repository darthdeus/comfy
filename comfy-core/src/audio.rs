use std::collections::hash_map::Entry;

use crate::*;

pub fn play_sound_ex(id: &str, params: PlaySoundParams) {
    play_sound_id_ex(sound_id(id), params);
}

pub fn play_sound_id_ex(sound: Sound, _params: PlaySoundParams) {
    play_sound_id(sound);
}

pub fn play_music_id_ex(sound: Sound, params: PlaySoundParams) {
    if params.looped {
        // TODO ??
        println!("looped music not supported yet");
    }
    play_sound_id(sound);
}

pub fn play_sound(id: &str) {
    play_sound_id(sound_id(id));
}

pub fn play_voice(id: &str) {
    play_sound_id(sound_id(id));
}

pub fn play_random_sound_ex(
    base_id: &str,
    amount: i32,
    settings: StaticSoundSettings,
) {
    let id = format!("{}-{}", base_id, gen_range(1, amount + 1));
    AudioSystem::play_sound(sound_id(&id), Some(settings), AudioTrack::None);
}

pub fn play_random_sound(base_id: &str, amount: i32) {
    let id = format!("{}-{}", base_id, gen_range(1, amount + 1));
    play_sound_id(sound_id(&id));
}

pub fn play_music(id: &str) {
    play_sound_id(sound_id(id));
}

pub fn play_sound_id(sound: Sound) {
    GLOBAL_STATE.borrow_mut().play_sound_queue.push(sound);
}

pub fn stop_sound(sound: &str) {
    stop_sound_id(sound_id(sound));
}

pub fn stop_sound_id(sound: Sound) {
    GLOBAL_STATE.borrow_mut().stop_sound_queue.push(sound);
}


#[derive(Copy, Clone, Debug, Default)]
pub struct PlaySoundParams {
    pub looped: bool,
}

pub struct PlaySoundCommand {
    pub sound: Sound,
    pub settings: StaticSoundSettings,
}

thread_local! {
    pub static AUDIO_SYSTEM: Lazy<RefCell<AudioSystem>> =
        Lazy::new(|| RefCell::new(AudioSystem::new()));
}

// pub static AUDIO_SYSTEM: Lazy<Mutex<AudioSystem>> =
//     Lazy::new(|| Mutex::new(AudioSystem::new()));

// unsafe impl Sync for AudioSystem {}
// unsafe impl Send for AudioSystem {}

pub enum AudioTrack {
    None,
    Filter,
}

pub struct AudioSystemImpl {
    pub manager: AudioManager,
    pub filter_track: TrackHandle,
    pub filter_handle: FilterHandle,

    pub master_volume: f64,
}

impl AudioSystemImpl {
    pub fn new(mut manager: AudioManager) -> Self {
        let mut builder = TrackBuilder::new();
        let filter_handle =
            builder.add_effect(FilterBuilder::new().cutoff(100.0));

        // builder.add_effect(ReverbBuilder::new().damping(0.8).feedback(0.9).mix(0.1));

        let filter_track =
            manager.add_sub_track(builder).expect("Failed to add filter track");

        Self { manager, filter_track, filter_handle, master_volume: 1.0 }
    }

    pub fn play_sound(
        &mut self,
        sound: Sound,
        settings: Option<StaticSoundSettings>,
        track: AudioTrack,
        // ) -> Option<impl DerefMut<Target = StaticSoundHandle>> {
    ) {
        let mut assets = ASSETS.borrow_mut();
        let assets = &mut *assets;

        let sounds = assets.sounds.lock();

        if let Some(mut sound_data) = sounds.get(&sound).cloned() {
            if let Some(settings) = settings {
                sound_data.settings = settings
                    .volume(kira::Volume::Amplitude(self.master_volume));

                match track {
                    AudioTrack::None => {}
                    AudioTrack::Filter => {
                        info!("filter track");
                        sound_data.settings = sound_data
                            .settings
                            .output_destination(&self.filter_track);
                    }
                }
            }


            match self.manager.play(sound_data) {
                Ok(handle) => {
                    match assets.sound_handles.entry(sound) {
                        Entry::Occupied(mut entry) => {
                            entry
                                .get_mut()
                                .stop(kira::tween::Tween::default())
                                .log_err();

                            entry.insert(handle);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(handle);
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to play sound: {:?}", err);
                }
            }
        } else {
            error!("No sound data for {:?}", sound);
        }
    }

    pub fn process_sounds(&mut self) {
        let _span = span!("process_sounds");

        let mut assets = ASSETS.borrow_mut();

        let stop_sound_queue =
            GLOBAL_STATE.borrow_mut().stop_sound_queue.drain(..).collect_vec();

        for sound in stop_sound_queue {
            match assets.sound_handles.entry(sound) {
                Entry::Occupied(mut entry) => {
                    entry
                        .get_mut()
                        .stop(kira::tween::Tween::default())
                        .log_err();
                    entry.remove();
                }
                Entry::Vacant(_) => {}
            }
        }

        let play_sound_queue =
            GLOBAL_STATE.borrow_mut().play_sound_queue.drain(..).collect_vec();

        for sound in play_sound_queue {
            let sound_data = assets.sounds.lock().get(&sound).cloned();

            if let Some(mut sound_data) = sound_data {
                sound_data.settings = StaticSoundSettings::new()
                    .volume(Volume::Amplitude(self.master_volume));

                match self.manager.play(sound_data) {
                    Ok(handle) => {
                        match assets.sound_handles.entry(sound) {
                            Entry::Occupied(mut entry) => {
                                entry
                                    .get_mut()
                                    .stop(kira::tween::Tween::default())
                                    .log_err();
                                entry.insert(handle);
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(handle);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error when playing sound: {:?}", err);
                    }
                }
            } else {
                error!("No sound data for {:?}", sound);
            }
        }
    }
}

pub struct AudioSystem {
    pub system: Option<AudioSystemImpl>,
}

impl AudioSystem {
    pub fn new() -> Self {
        // AudioManager::<kira::manager::backend::mock::MockBackend>::new(AudioManagerSettings::default())
        let manager =
            AudioManager::<kira::manager::backend::cpal::CpalBackend>::new(
                AudioManagerSettings::default(),
            )
            .map_err(|err| {
                error!("Failed to initialize audio manager: {:?}", err);
                err
            })
            .ok();

        let system = manager.map(AudioSystemImpl::new);

        Self { system }
    }

    // pub fn get() -> impl Deref<Target = Self> {
    //     AUDIO_SYSTEM.lock()
    // }
    //
    // pub fn get_mut() -> impl DerefMut<Target = Self> {
    //     AUDIO_SYSTEM.lock()
    // }

    pub fn process_sounds() {
        AUDIO_SYSTEM.with(|audio| {
            if let Some(system) = audio.borrow_mut().system.as_mut() {
                system.process_sounds();
            }
        });

        // let mut audio = AudioSystem::get_mut();
        // if let Some(system) = audio.system.as_mut() {
        //     system.process_sounds();
        // }
    }

    pub fn play_sound(
        sound: Sound,
        settings: Option<StaticSoundSettings>,
        track: AudioTrack,
    ) {
        AUDIO_SYSTEM.with(|audio| {
            if let Some(system) = audio.borrow_mut().system.as_mut() {
                system.play_sound(sound, settings, track);
            }
        });

        // let mut audio = AudioSystem::get_mut();
        // if let Some(system) = audio.system.as_mut() {
        //     system.play_sound(sound, settings, track);
        // }
    }
}
