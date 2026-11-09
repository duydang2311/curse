use std::{
    ffi::{OsStr, c_void},
    iter::once,
    mem,
    os::windows::ffi::OsStrExt,
    path::PathBuf,
    ptr,
    sync::mpsc,
};

use daemon::{
    ma_decoder, ma_decoder_init_file_w, ma_decoder_read_pcm_frames, ma_decoder_uninit, ma_device,
    ma_device_config_init, ma_device_init, ma_device_start, ma_device_type_ma_device_type_playback,
    ma_device_uninit, ma_result_MA_SUCCESS,
};
use eyre::bail;

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
        println!("drop");
        unsafe {
            ma_device_uninit(&mut *self.device);
            ma_decoder_uninit(&mut *self.decoder);
        }
    }
}

pub fn run_audio_thread(rx: mpsc::Receiver<AudioMsg>) {
    let mut playback: Option<Playback> = None;
    loop {
        match rx.recv() {
            Ok(AudioMsg::Play(path_buf)) => match play(&path_buf) {
                Ok(pb) => {
                    playback = Some(pb);
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
}

fn to_wide(s: &OsStr) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn play(path_buf: &PathBuf) -> eyre::Result<Playback> {
    let mut decoder: Box<ma_decoder> = Box::new(unsafe { mem::zeroed() });
    let result = unsafe {
        ma_decoder_init_file_w(
            to_wide(path_buf.as_os_str()).as_ptr(),
            ptr::null(),
            &mut *decoder,
        )
    };
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
    input: *const c_void,
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
