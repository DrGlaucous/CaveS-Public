
//use libxmp_lite::LibXmpPlayer;

extern "C" {
    fn add_nums_proxy(a: i32, b: i32) -> i32;
}

fn main() {

    unsafe{
        let ttt = add_nums_proxy(4,6);
        println!("world, {}", ttt);
    }
    


}
