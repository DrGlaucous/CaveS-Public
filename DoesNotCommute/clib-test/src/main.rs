
use std::fs::{read, File};
use std::io::prelude::*;

use libxmp_lite::LibXmpPlayer;


// extern "C" {
//     fn add_nums_proxy(a: i32, b: i32) -> i32;
// }



fn main() {


    //load file
    if let Ok(mut ff) = File::open("beyond_music.mod") {
        let mut buff = Vec::new();
        let _ = ff.read_to_end(&mut buff);

        let mut context = LibXmpPlayer::new(&buff, 48000).unwrap();

        if let Some(name) = context.get_name() {
            println!("{}", name.as_str());
        }

        let mut samples: [i16; 48] = [0;48];

        context.get_buffer(&mut samples);

        context.get_buffer(&mut samples);

        context.get_buffer(&mut samples);

    }
    

    // unsafe{
    //     let ttt = add_nums_proxy(4,6);
    //     println!("world, {}", ttt);
    // }
    


}
