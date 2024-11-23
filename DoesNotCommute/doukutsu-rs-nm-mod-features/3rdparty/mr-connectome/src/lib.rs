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
    //how large the weight must be before firing
    threshold: i32,

    //list of outputs to fire to
    outputs: Vec<KeyWeight>,
}

struct NeuronWeight {
    //current weight of the neuron
    weight: [i32; 2],

    //how long we wait until zeroing the weight variable
    persistence: i32,

    //current time before we zero the weight variable, incremented each frame, or reset to 0
    time: i32,
}

pub struct Connectome {

    //contains neuron threshhold and a list of connected neurons
    neuron_info: HashMap<String, NeuronInfo>,

    //the current accumulated weight of a neuron, used to determine if it should fire or not.
    //to appease the borrow checker, this needs to be seperate from the NeuronInfo
    neuron_weight: HashMap<String, NeuronWeight>,

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

        let mut neuron_info: HashMap<String, NeuronInfo> =  HashMap::new();
        let mut neuron_weight: HashMap<String, NeuronWeight> =  HashMap::new();

        //read CSV data
        let mut reader = Reader::from_reader(data);
        for result in reader.records() {
            let record = result.unwrap();
            
            if record.len() < 3 {
                continue;
            }

            let neuron = format!("{}", &record[0]);
            let output = format!("{}", &record[1]);
            let weight = record[2].parse::<i32>().unwrap();

            //if neuron already exists, add this connection to its list
            if let Some(neu) = neuron_info.get_mut(&neuron) {
                neu.outputs.push(KeyWeight{
                    key: output,
                    weight,
                });
            } else {
                //make new neuron and weight entry

                let outputs: Vec<KeyWeight> =  vec![KeyWeight{
                    key: output,
                    weight: weight,
                }];

                neuron_info.insert(neuron.clone(), NeuronInfo{
                    threshold: 30, //default threshhold size
                    outputs,
                });

                neuron_weight.insert(neuron, NeuronWeight{
                    weight: [0,0],
                    persistence: 10, //default for now
                    time: 0,
                });

            }


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
            let n_weight = self.neuron_weight.get_mut(nkey).unwrap();
            if n_weight.weight[self.weight_index] > neuron.threshold {

                //reset weight because we fired
                n_weight.weight[self.weight_index] = 0;

                //for each child
                for connection in &neuron.outputs {
                    //update weight
                    if let Some(child_weight) = self.neuron_weight.get_mut(&connection.key) {
                        child_weight.weight[self.next_weight_index] += connection.weight; //at this point, this variable represents the delta weight from last tick
                        child_weight.time = 0; //reset fire timeout
                    }
                }
            }
        }

        //update persistence timeout weight as needed, and make the next weight absolute
        for (_, weight) in self.neuron_weight.iter_mut() {

            if weight.time > weight.persistence && weight.persistence >= 0 {
                weight.weight[self.weight_index] = 0; //reset weight
                weight.time = 0; //reset fire timeout
            } else {
                //make the next weight index absolute
                weight.weight[self.next_weight_index] += weight.weight[self.weight_index];
            }
            weight.time += 1;

        }

        //flip index
        let intermediate = self.weight_index;
        self.weight_index = self.next_weight_index;
        self.next_weight_index = intermediate;


    }

    //returns True if the neuron was found and weight adjusted
    pub fn add_weight_to_neuron(&mut self, key: &String, weight: i32) -> bool {

        if let Some(child_weight) = self.neuron_weight.get_mut(key) {
            child_weight.weight[self.next_weight_index] = child_weight.weight[self.weight_index] + weight;
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
                    child_weight.weight[self.next_weight_index] = child_weight.weight[self.weight_index] + connection.weight;
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





