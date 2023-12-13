use std::fmt;
use std::io;

use crate::sound::wav;

#[derive(Clone)]
pub struct SoundBank {
    pub wave100: Box<[u8; 25600]>,

    pub samples: Vec<wav::WavSample>,
}

impl fmt::Display for SoundBank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "WAVE100: {:2X?}...", &self.wave100[..8])?;

        for sample in &self.samples {
            writeln!(f, "{}", sample)?;
        }

        Ok(())
    }
}

impl SoundBank {
    pub fn load_from<R: io::Read>(mut f: R) -> io::Result<SoundBank> {

        //100 waves with 256 samples each? (yes)
        let mut wave100 = Box::new([0u8; 25600]);

        //grab the wave table (this is of a known size, so we can just snatch it all right here)
        f.read_exact(wave100.as_mut())?;

        //holds the wave samples
        //is it 43 or 42 waves? it's 42, windows explorer lied to me.
        let mut samples = Vec::with_capacity(42); //16 originally

        loop {
            match wav::WavSample::read_from(&mut f) {
                Ok(sample) => {
                    log::info!("Loaded sample: {:?}", sample.format);
                    samples.push(sample)
                }
                Err(err) => {
                    log::error!("Failed to read next sample: {}", err);
                    return Ok(SoundBank { wave100, samples });
                }
            }
        }
    }

    pub fn get_wave(&self, index: usize) -> &[u8] {
        &self.wave100[index * 256..(index + 1) * 256]
    }
}
