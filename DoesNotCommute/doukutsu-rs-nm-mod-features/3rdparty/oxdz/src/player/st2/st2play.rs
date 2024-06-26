use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::stm::StmData;
use mixer::Mixer;

/// Scream Tracker 2 replayer
///
/// An oxdz player based on st2play written by Sergei "x0r" Kolzun, ported
/// to Rust by Claudio Matsuoka.


//const ST2BASEFREQ      : u32 = 36072500;  // 2.21
//const ST2BASEFREQ      : u32 = 35468950;  // 2.3

const FXMULT           : u16 =  0x0a;

//                Pattern Command Bytes
//                 (info bytes are in hex)
// A - Set tempo to [INFO]. 60 normal.
// B - Break pattern and jmp to order [INFO]
// C - Break pattern
// D - Slide volume; Hi-nibble=up, Lo-nibble=down
// E - Slide down at speed [INFO]
// F - Slide up at speed [INFO]
// G - Slide to the note specified at speed [INFO]
// H - Vibrato; Hi-nibble, speed (bigger is faster)
//              Lo-nibble, size (bigger is bigger)
// I - Tremor; Hi-nibble, ontime
//             Lo-nibble, offtime
// J - Arpeggio; inoperative at the moment

// LMN can be entered in the editor but don't do anything
const FX_SPEED         : u16 = 0x01;
const FX_POSITIONJUMP  : u16 = 0x02;
const FX_PATTERNBREAK  : u16 = 0x03;
const FX_VOLUMESLIDE   : u16 = 0x04;
const FX_PORTAMENTODOWN: u16 = 0x05;
const FX_PORTAMENTOUP  : u16 = 0x06;
const FX_TONEPORTAMENTO: u16 = 0x07;
const FX_VIBRATO       : u16 = 0x08;
const FX_TREMOR        : u16 = 0x09;
//const FX_ARPEGGIO      : u16 = 0x0a;
//const FX_VIBRA_VSLIDE  : u16 = 0x0b;
//const FX_TONE_VSLIDE   : u16 = 0x0f;

lazy_static! {
    static ref TEMPO_MUL: Box<[u16; 18]> = Box::new([ 140, 50, 25, 15, 10, 7, 6, 4, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1 ]);

    static ref PERIOD_TABLE: Box<[i16; 16*5]> = Box::new([
        17080, 16012, 15184, 14236, 13664, 12808, 12008, 11388, 10676, 10248,  9608,  9108, 0, 0, 0, 0,
         8540,  8006,  7592,  7118,  6832,  6404,  6004,  5694,  5338,  5124,  4804,  4554, 0, 0, 0, 0,
         4270,  4003,  3796,  3559,  3416,  3202,  3002,  2847,  2669,  2562,  2402,  2277, 0, 0, 0, 0,
         2135,  2001,  1898,  1779,  1708,  1601,  1501,  1423,  1334,  1281,  1201,  1138, 0, 0, 0, 0,
         1067,  1000,   949,   889,   854,   800,   750,   711,   667,   640,   600,   569, 0, 0, 0, 0 
    ]);

    static ref LFO_TABLE: Box<[i16; 65]> = Box::new([
           0,   24,   49,   74,   97,  120,  141,  161,  180,  197,  212,  224,  235,  244,  250,  253,
         255,  253,  250,  244,  235,  224,  212,  197,  180,  161,  141,  120,   97,   74,   49,   24,
           0,  -24,  -49,  -74,  -97, -120, -141, -161, -180, -197, -212, -224, -235, -244, -250, -253,
        -255, -253, -250, -244, -235, -224, -212, -197, -180, -161, -141, -120,  -97,  -74,  -49,  -24,
           0
    ]);
}


#[derive(SaveRestore)]
pub struct St2Play {
    options         : Options,

    //sample_rate     : u16,
    pattern_current : u16,
    change_pattern  : bool,
    current_tick    : u16,
    ticks_per_row   : u16,
    //current_frame   : u16,
    //frames_per_tick : u16,
    tempo_factor    : u16,  // not in st2play
    loop_count      : u16,
    order_first     : u16,
    order_next      : u16,
    order_current   : u16,
    tempo           : u8,
    global_volume   : u8,
    //play_single_note: u8,
    //uint8_t *order_list_ptr;
    //uint8_t *pattern_data_ptr;
    //st2_channel_t channels[4];
    //st2_sample_t samples[32];

    channels: [St2Channel; 4],
}

impl St2Play {
    pub fn new(_module: &Module, options: Options) -> Self {
        St2Play {
            options,
            //sample_rate     : 15909,
            pattern_current : 0,
            change_pattern  : false,
            current_tick    : 0,
            ticks_per_row   : 0,
            //current_frame   : 1,
            //frames_per_tick : 1,
            tempo_factor    : 0,
            loop_count      : 0,
            order_first     : 0,
            order_next      : 0,
            order_current   : 0,
            tempo           : 0x60,
            global_volume   : 64,
            //play_single_note: 0,
            channels        : [St2Channel::new(); 4],
        }
    }

    fn set_tempo(&mut self, tempo: u16) {
        self.ticks_per_row = tempo >> 4;
        //self.frames_per_tick = self.sample_rate / (49 - ((TEMPO_MUL[self.ticks_per_row as usize] * (tempo & 0x0f)) >> 4));
        self.tempo_factor = 49 - ((TEMPO_MUL[self.ticks_per_row as usize] * (tempo & 0x0f)) >> 4);
    }

/*
    fn update_frequency(&mut self, chn: usize) {
        let mut step = 0_u32;
        let ch = &mut self.channels[chn];

        if ch.period_current >= 551 {
            let temp = ST2BASEFREQ / ch.period_current as u32;
            step = ((temp / self.sample_rate as u32) & 0xffff) << 16;
            step |= (((temp % self.sample_rate as u32) << 16) / self.sample_rate as u32) & 0xffff;
        }

        ch.smp_step = step;
    }
*/

    fn cmd_once(&mut self, chn: usize) {
        let cmd = self.channels[chn].event_cmd;
        let infobyte = self.channels[chn].event_infobyte;

        match cmd {
            FX_SPEED => if infobyte != 0 {
                self.set_tempo(infobyte);
            },
            FX_POSITIONJUMP => {
                self.order_next = infobyte;
            },
            FX_PATTERNBREAK => {
                self.change_pattern = true;
            },
            _ => {},
        }
    }

    fn cmd_tick(&mut self, chn: usize) {
        let cmd = self.channels[chn].event_cmd;
        let infobyte = self.channels[chn].event_infobyte;
        let ch = &mut self.channels[chn];

        match cmd  {
            FX_VOLUMESLIDE => {
                if infobyte & 0x0f != 0 {
                    ch.volume_current -= (infobyte & 0x0f) as i16;
                    if ch.volume_current <= -1 {
                        ch.volume_current = 0;
                    }
                } else {
                    ch.volume_current += (infobyte >> 4) as i16;
                    if ch.volume_current >= 65 {
                        ch.volume_current = 64;
                    }
                }
            },
            FX_PORTAMENTODOWN => {
                ch.period_current += (FXMULT * infobyte) as i16;
                //self.update_frequency(chn);
            }, 
            FX_PORTAMENTOUP => {
                ch.period_current -= (FXMULT * infobyte) as i16;
                //self.update_frequency(chn);
            },
            FX_TREMOR => {
                if ch.tremor_counter == 0 {
                    if ch.tremor_state == 1 {
                        ch.tremor_state = 0;
                        ch.volume_current = 0;
                        ch.tremor_counter = infobyte as u16 & 0x0f;
                    } else {
                        ch.tremor_state = 1;
                        ch.volume_current = ch.volume_initial;
                        ch.tremor_counter = infobyte as u16 >> 4;
                    }
                } else {
                    ch.tremor_counter -= 1;
                }
            },
            _ => {
                ch.tremor_counter = 0;
                ch.tremor_state = 1;
                match cmd {
                    FX_TONEPORTAMENTO => {
                        if ch.period_current != ch.period_target {
                            if ch.period_current > ch.period_target {
                                ch.period_current -= (FXMULT * infobyte) as i16;
                                if ch.period_current < ch.period_target {
                                    ch.period_current = ch.period_target;
                                }
                            } else {
                                ch.period_current += (FXMULT * infobyte) as i16;
                                if ch.period_current > ch.period_target {
                                    ch.period_current = ch.period_target;
                                }
                            }
                            //self.update_frequency(chn);
                        }
                    },
                    FX_VIBRATO => {
                        ch.period_current = (FXMULT as i16 * ((LFO_TABLE[ch.vibrato_current as usize >> 1] *
                                            (infobyte & 0x0f) as i16) >> 6)) + ch.period_target;
                        //self.update_frequency(chn);
                        ch.vibrato_current = (ch.vibrato_current + ((infobyte >> 4) << 1)) & 0x7e;
                    },
                    _ => {
                        ch.vibrato_current = 0;
                    },
                }
            },
        }
    }

    fn trigger_note(&mut self, chn: usize, module: &StmData, mixer: &mut Mixer) {
        let note = self.channels[chn].event_note as usize;
        let volume = self.channels[chn].event_volume as i16;
        let smp = self.channels[chn].event_smp as usize;
        let cmd = self.channels[chn].event_cmd;

        if self.channels[chn].event_volume != 65 {
            self.channels[chn].volume_current = volume;
            self.channels[chn].volume_initial = self.channels[chn].volume_current;
        }

        if cmd == FX_TONEPORTAMENTO {
            if note != 255 {
                self.channels[chn].period_target = PERIOD_TABLE[note];
            }
            return;
        }

        if smp != 0 {
            let instrument = &module.instruments[smp - 1];

            //self.channels[chn].smp_name = self.samples[smp].name;
            if volume == 65 {
                self.channels[chn].volume_current = (instrument.volume & 0xff) as i16;
                self.channels[chn].volume_initial = self.channels[chn].volume_current;
            }

            //self.channels[chn].smp_data_ptr = ctx->samples[smp].data;
            mixer.set_sample(chn, smp);

            if module.instruments[smp-1].loop_end != 0xffff {
                //self.channels[chn].smp_loop_end = ctx->samples[smp].loop_end;
                //self.channels[chn].smp_loop_start = ctx->samples[smp].loop_start;
                mixer.set_loop_start(chn, module.instruments[smp-1].loop_start as u32);
                mixer.set_loop_end(chn, module.instruments[smp-1].loop_end as u32);
                mixer.enable_loop(chn, true);
            } else {
                //self.channels[chn].smp_loop_end = ctx->samples[smp].length;
                //self.channels[chn].smp_loop_start = 0xffff;
                mixer.enable_loop(chn, false);
            }
        }

        if note != 255 {
            //self.channels[chn].smp_position = 0;
            mixer.set_voicepos(chn, 0.0);

            if note == 254 {
                //self.channels[chn].smp_loop_end = 0;
                //self.channels[chn].smp_loop_start = 0xffff;
                mixer.enable_loop(chn, false);
            } else {
                //self.channels[chn].volume_meter = self.channels[chn].volume_current >> 1;
                //self.channels[chn].period_current = PERIOD_TABLE[note] * 8448 / ctx->samples[smp].c2spd; /* 8448 - 2.21; 8192 - 2.3 */
                self.channels[chn].period_current = PERIOD_TABLE[note];
                self.channels[chn].period_target = self.channels[chn].period_current;
                //self.update_frequency(chn);
            }
        }

        self.cmd_once(chn);
    }

    fn process_row(&mut self, chn: usize, module: &StmData, mut mixer: &mut Mixer) {
        self.channels[chn].row += 1;
        if self.channels[chn].row >= 64 {
            self.change_pattern = true;
        }

        //if self.channels[chn].on {
            {
                let row = self.channels[chn].row;
                let ch = &mut self.channels[chn];
                let event = module.patterns.event(self.pattern_current, row - 1, chn);

                ch.event_note     = event.note as u16;
                ch.event_smp      = event.smp as u16;
                ch.event_volume   = event.volume;
                ch.event_cmd      = event.cmd as u16;
                ch.event_infobyte = event.infobyte as u16;
            }
            self.trigger_note(chn, &module, &mut mixer);

            if self.channels[chn].event_cmd == FX_TREMOR {
                self.cmd_tick(chn);
            }
        //}
    }

    fn change_pattern(&mut self, module: &StmData) {
        let pat = module.orders[self.order_next as usize];
        if pat == 98 || pat == 99 {
            self.order_next = if pat == 99 { self.order_first } else { 0 };
            self.loop_count += 1;
        }

	// oxdz: sanity check
        if self.order_next as usize >= module.len() {
            self.order_next = 0;
        }

        self.pattern_current = module.orders[self.order_next as usize] as u16;
//      self.order_list_ptr[self.order_next] = 99;
        self.order_current = self.order_next;
        self.order_next += 1;

        for ch in &mut self.channels {
            ch.row = 0;
        }
    }

    fn process_tick(&mut self, module: &StmData, mut mixer: &mut Mixer) {
        if self.current_tick != 0 {
            self.current_tick -= 1;
            for i in 0..4 {
                self.cmd_tick(i);
            }
        } else {
            //if !ctx->play_single_note {
                if self.change_pattern {
                    self.change_pattern = false;
                    self.change_pattern(&module);
                }

                for i in 0..4 {
                    self.process_row(i, &module, &mut mixer);
                }

                self.current_tick = if self.ticks_per_row != 0 { self.ticks_per_row - 1 } else { 0 };
            //}
        }

        for i in 0..4 {
            self.channels[i].volume_mix = (self.channels[i].volume_current as u16 * self.global_volume as u16) >> 6;
        }
    }
}


#[derive(Default,Copy,Clone)]
struct St2Channel {
    //on               : bool,
    //empty            : bool,
    row              : u16,
    //pattern_data_offs: usize,
    event_note       : u16,
    event_volume     : u8,
    event_smp        : u16,
    event_cmd        : u16,
    event_infobyte   : u16,
    //last_note        : u16,
    period_current   : i16,
    period_target    : i16,
    vibrato_current  : u16,
    tremor_counter   : u16,
    tremor_state     : u16,
    //uint8_t *smp_name;
    //uint8_t *smp_data_ptr;
    //uint16_t smp_loop_end;
    //uint16_t smp_loop_start;
    //uint16_t smp_c2spd;
    //uint32_t smp_position;
    //smp_step         : u32,
    volume_initial   : i16,
    volume_current   : i16,
    //uint16_t volume_meter;
    volume_mix       : u16,
}

impl St2Channel {

    pub fn new() -> Self {
        Default::default()
    }
}


impl FormatPlayer for St2Play {
    fn start(&mut self, data: &mut PlayerData, mdata: &dyn ModuleData, mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<StmData>().unwrap();

        self.tempo = 0x60;
        self.order_next = 0;

        // sr/x = (sr*250)/(T*100) => T = 25*x/10
        data.tempo = self.tempo_factor as f32;
        data.speed = module.speed as usize;
        data.time  = 0.0;

        data.initial_speed = data.speed;
        data.initial_tempo = data.tempo;

        let t = self.tempo as u16;
        self.set_tempo(t);
        //self.current_frame = self.frames_per_tick;
        self.change_pattern(&module);

        let pan = match self.options.option_int("pan") {
            Some(val) => val,
            None      => 70,
        };
        let panl = -128 * pan / 100;
        let panr = 127 * pan / 100;

        mixer.set_pan(0, panl);
        mixer.set_pan(1, panr);
        mixer.set_pan(2, panr);
        mixer.set_pan(3, panl);

    }

    fn play(&mut self, data: &mut PlayerData, mdata: &dyn ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<StmData>().unwrap();

        self.process_tick(&module, &mut mixer);
        for chn in 0..4 {
            let ch = &mut self.channels[chn];
            mixer.set_period(chn, (ch.period_current / FXMULT as i16) as f64);
            if ch.volume_current != 65 {
                mixer.set_volume(chn, ch.volume_mix as usize * 16);
            }
        }

        data.frame = ((self.ticks_per_row - self.current_tick) % self.ticks_per_row) as usize;

        data.row = self.channels[0].row as usize;
        if data.frame > 0 { data.row -= 1 }
        data.row %= 64;

        data.pos = self.order_next as usize;
        if data.row > 0 || data.frame > 0 { data.pos -= 1 }
        data.pos %= mdata.len();

        data.speed = self.ticks_per_row as usize;
        data.tempo = 2.5 * self.tempo_factor as f32;
        data.time += 20.0 * 125.0 / data.tempo;
    }

    fn reset(&mut self) {
    }

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state)
    }
}
