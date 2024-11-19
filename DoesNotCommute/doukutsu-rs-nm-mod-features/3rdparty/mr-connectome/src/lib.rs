use std::collections::HashMap;
use std::ffi::{c_short, c_uint, c_void, CStr};
use std::io;
use std::io::ErrorKind;
use std::mem::{MaybeUninit, size_of};
use std::os::raw::{c_char, c_int, c_long, c_uchar};
use std::ptr::null_mut;
use std::marker::{Sync, Send};


extern crate csv;
use csv::Reader;



//how much the neuron at [key] should be affected by [weight] when this neuron fires
struct KeyWeight {
    pub key: String,
    pub weight: i32,
}

struct NeuronInfo {
    threshold: i32,
    outputs: Vec<KeyWeight>,
}


pub struct Connectome {

    //contains neuron threshhold and a list of connected neurons
    neuron_info: HashMap<String, NeuronInfo>,

    //the current accumulated weight of a neuron, used to determine if it should fire or not.
    //to appease the borrow checker, this needs to be seperate from the NeuronInfo
    neuron_weight: HashMap<String, [i32; 2]>,

    //which index of the neuron_weight will be used for refrence vs accumulating new weight
    //we these use two indicies so that all the neurons have the chance to update from the same static state.
    weight_index: usize,
    next_weight_index: usize,

}


impl Connectome {

    //create a new connectome from a csv-formatted file buffer
    pub fn new(linkage_data: Option<&[u8]>) -> Connectome {

        //either use external data or hard-baked data. (if someone decides to design a "better brain")
        let data = if let Some(data) = linkage_data {
            data
        } else {
            include_bytes!("data/CElegansConnectome.csv")
        };


        //read CSV data
        let mut reader = Reader::from_reader(data);
        for result in reader.records() {
            let record = result.unwrap();
            println!("{:?}", record);
        }


        return Connectome{
            neuron_info: HashMap::new(),
            neuron_weight: HashMap::new(),
            weight_index: 0,
            next_weight_index: 1,
        }
    }

    //updates all the neurons 
    pub fn tick(&mut self) {

        //update each neuron
        for (nkey, neuron) in self.neuron_info.iter() {

            //check if current neuron weight surpasses its threshhold. If it does, "fire" the neuron and update child weights
            if self.neuron_weight.get(nkey).unwrap()[self.weight_index] > neuron.threshold {
                //for each child
                for connection in &neuron.outputs {
                    //update weight
                    if let Some(child_weight) = self.neuron_weight.get_mut(&connection.key) {
                        child_weight[self.next_weight_index] = child_weight[self.weight_index] + connection.weight;
                    }
                }
            }
        }

        //flip index
        let intermediate = self.weight_index;
        self.weight_index = self.next_weight_index;
        self.next_weight_index = intermediate;

    }

    //returns True if the neuron was found and weight adjusted
    pub fn add_weight_to_neuron(&mut self, key: &String, weight: i32) -> bool {

        if let Some(child_weight) = self.neuron_weight.get_mut(key) {
            child_weight[self.next_weight_index] = child_weight[self.weight_index] + weight;
            return true;
        }

        false
    }

    //fire this neuron, but do not zero its weight
    pub fn stimulate_neuron(&mut self, key: &String) -> bool {

        //get this neuron
        if let Some(neuron) = self.neuron_info.get(key) {
            //for each child
            for connection in &neuron.outputs {
                //update weight
                if let Some(child_weight) = self.neuron_weight.get_mut(&connection.key) {
                    child_weight[self.next_weight_index] = child_weight[self.weight_index] + connection.weight;
                }
            }
            return true;
        }

        false
    }

    //pre-baked stimulation patterns

    //stimulates nearby food
    pub fn stimulate_food(&mut self) {
        let _ = self.stimulate_neuron(&format!("ADFL"));
        let _ = self.stimulate_neuron(&format!("ADFR"));
        let _ = self.stimulate_neuron(&format!("ASGR"));
        let _ = self.stimulate_neuron(&format!("ASGL"));
        let _ = self.stimulate_neuron(&format!("ASIL"));
        let _ = self.stimulate_neuron(&format!("ASIR"));
        let _ = self.stimulate_neuron(&format!("ASJR"));
        let _ = self.stimulate_neuron(&format!("ASJL"));
    }

    //stimulates hitting a wall head-on
    pub fn stimulate_bump(&mut self) {
        let _ = self.stimulate_neuron(&format!("FLPR"));
        let _ = self.stimulate_neuron(&format!("FLPL"));
        let _ = self.stimulate_neuron(&format!("ASHL"));
        let _ = self.stimulate_neuron(&format!("ASHR"));
        let _ = self.stimulate_neuron(&format!("IL1VL"));
        let _ = self.stimulate_neuron(&format!("IL1VR"));
        let _ = self.stimulate_neuron(&format!("OLQDL"));
        let _ = self.stimulate_neuron(&format!("OLQDR"));
        let _ = self.stimulate_neuron(&format!("OLQVR"));
        let _ = self.stimulate_neuron(&format!("OLQVL"));
    }

    //not sure where on the worm this corresponds to. I think anterior is anywhere on the body towards the front, posterior is towards the back?
    pub fn stimulate_anterior_harsh_touch(&mut self) {
        let _ = self.stimulate_neuron(&format!("FLPL"));
        let _ = self.stimulate_neuron(&format!("FLPR"));
        let _ = self.stimulate_neuron(&format!("BDUL"));
        let _ = self.stimulate_neuron(&format!("BDUR"));
        let _ = self.stimulate_neuron(&format!("SDQR"));
    }

    pub fn stimulate_posterior_harsh_touch(&mut self) {
        let _ = self.stimulate_neuron(&format!("PVDL"));
        let _ = self.stimulate_neuron(&format!("PVDR"));
        let _ = self.stimulate_neuron(&format!("PVCL"));
        let _ = self.stimulate_neuron(&format!("PVCR"));
    }




}





