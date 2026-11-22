use common::msg::ServerMsg;
use daemon::{
    ma_decoder, ma_decoder_read_pcm_frames, ma_decoder_uninit, ma_device, ma_device_config_init,
    ma_device_init, ma_device_start, ma_device_type_ma_device_type_playback, ma_device_uninit,
    ma_result, ma_result_MA_SUCCESS,
};
use eyre::bail;
use std::{
    ffi::c_void,
    mem,
    path::{Path, PathBuf},
    ptr,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crate::app::App;

pub enum AudioMsg {
    Play(PathBuf),
    Shutdown,
}

pub struct Playback {
    decoder: Box<ma_decoder>,
    device: Box<ma_device>,
}

impl Drop for Playback {
    fn drop(&mut self) {
        unsafe {
            ma_device_uninit(&mut *self.device);
            ma_decoder_uninit(&mut *self.decoder);
        }
    }
}

pub fn spawn_audio_thread(app: App, rx: mpsc::Receiver<AudioMsg>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut playback: Option<Playback> = None;
        loop {
            match rx.recv() {
                Ok(AudioMsg::Play(path_buf)) => match play(&path_buf) {
                    Ok(pb) => {
                        playback = Some(pb);
                        app.clients.broadcast(&ServerMsg::TrackStarted(path_buf));
                    }
                    Err(e) => {
                        eprintln!("failed to play {}: {}", &path_buf.display(), e);
                    }
                },
                Ok(AudioMsg::Shutdown) => break,
                Err(e) => {
                    eprintln!("failed to receive audio msg: {}", e);
                }
            }
        }
    })
}

#[cfg(windows)]
fn init_decoder(path: &Path, decoder: &mut ma_decoder) -> eyre::Result<ma_result> {
    use daemon::ma_decoder_init_file_w;
    use std::{iter::once, os::windows::ffi::OsStrExt};

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    Ok(unsafe { ma_decoder_init_file_w(wide.as_ptr(), ptr::null(), &mut *decoder) })
}

#[cfg(unix)]
fn init_decoder(path: &Path, decoder: &mut ma_decoder) -> eyre::Result<ma_result> {
    use daemon::ma_decoder_init_file;
    use std::{ffi::CString, os::unix::ffi::OsStrExt};

    let c_path = CString::new(path.as_os_str().as_bytes())?;
    Ok(unsafe { ma_decoder_init_file(c_path.as_ptr(), ptr::null(), &mut *decoder) })
}

fn play(path: &Path) -> eyre::Result<Playback> {
    let mut decoder: Box<ma_decoder> = Box::new(unsafe { mem::zeroed() });
    let result = init_decoder(path, &mut decoder)?;
    if result != ma_result_MA_SUCCESS {
        bail!("failed to load file: code {}", result)
    }

    let mut device_config =
        unsafe { ma_device_config_init(ma_device_type_ma_device_type_playback) };
    device_config.playback.format = decoder.outputFormat;
    device_config.playback.channels = decoder.outputChannels;
    device_config.sampleRate = decoder.outputSampleRate;
    device_config.dataCallback = Some(data_callback);
    device_config.pUserData = &mut *decoder as *mut ma_decoder as *mut c_void;

    let mut device: Box<ma_device> = Box::new(unsafe { mem::zeroed() });
    if unsafe { ma_device_init(ptr::null_mut(), &device_config, &mut *device) }
        != ma_result_MA_SUCCESS
    {
        unsafe {
            ma_decoder_uninit(&mut *decoder);
        }
        bail!("failed to open playback device");
    }

    if unsafe { ma_device_start(&mut *device) } != ma_result_MA_SUCCESS {
        unsafe {
            ma_device_uninit(&mut *device);
            ma_decoder_uninit(&mut *decoder);
        }
        bail!("failed to start playback device");
    }
    Ok(Playback { decoder, device })
}

extern "C" fn data_callback(
    device: *mut ma_device,
    output: *mut c_void,
    _input: *const c_void,
    frame_count: u32,
) {
    let decoder = unsafe { (*device).pUserData as *mut ma_decoder };
    if decoder.is_null() {
        return;
    }

    unsafe {
        ma_decoder_read_pcm_frames(decoder, output, frame_count as u64, ptr::null_mut());
    }
}
