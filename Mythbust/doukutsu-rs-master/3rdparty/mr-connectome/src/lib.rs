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
#[derive(Debug, Clone)]
pub struct KeyWeight {
    pub key: String,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct NeuronInfo {
    //how large the weight must be before firing
    threshold: i32,

    //list of outputs to fire to
    outputs: Vec<KeyWeight>,
}

#[derive(Debug, Clone)]
pub struct NeuronWeight {
    //current weight of the neuron
    //pub weight: [i32; 2],

    pub refrence_weight: i32, //value refrenced when firing the neuron
    pub new_weight: i32, //value new weights are added to (this is later pushed to refrence_weight)

    //how long we wait until zeroing the weight variable
    pub persistence: i32,

    //current time before we zero the weight variable, incremented each frame, or reset to 0
    pub time: i32,
}


#[derive(Debug, Clone)]
pub struct Connectome {

    //contains neuron threshhold and a list of connected neurons
    neuron_info: HashMap<String, NeuronInfo>,

    //the current accumulated weight of a neuron, used to determine if it should fire or not.
    //to appease the borrow checker, this needs to be seperate from the NeuronInfo
    neuron_weight: HashMap<String, NeuronWeight>,

    //which index of the neuron_weight will be used for refrence vs accumulating new weight
    //we these use two indicies so that all the neurons have the chance to update from the same static state.
    //weight_index: usize,
    //next_weight_index: usize,

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
            
            //get real count of non-empty entries (I.E. not ""), since some programs make an even CSV grid even if the majority of the spaces contain nothing
            let real_record_count = record.iter()
            .filter(|a| *a != "").count();

            //need at least neuron, persistence, threshold
            if real_record_count < 3 {
                continue;
            }

            let neuron = format!("{}", &record[0]);
            let persistence = record[1].parse::<i32>().unwrap();
            let threshold = record[2].parse::<i32>().unwrap();

            //number of outgoing connections
            let connection_count = (real_record_count - 3) / 2;

            //add each outgoing connection to the output vector
            let mut outputs: Vec<KeyWeight> =  Vec::new();
            for a in 0..connection_count {
                let idx = (a * 2) + 3;

                let out_neuron = format!("{}", &record[idx]);
                let out_weight = record[idx + 1].parse::<i32>().unwrap();

                outputs.push(KeyWeight{
                    key: out_neuron,
                    weight: out_weight,
                });
            }


            //test
            // for a in &outputs {
            //     println!("{} {}", a.key, a.weight);
            // }

            neuron_info.insert(neuron.clone(), NeuronInfo{
                threshold,
                outputs,
            });

            neuron_weight.insert(neuron, NeuronWeight{
                //weight: [0,0], //current weight (starts at 0)
                refrence_weight: 0,
                new_weight: 0,

                persistence,
                time: 0, //current time left (starts at 0)
            });


        }


        return Connectome{
            neuron_info,//: HashMap::new(),
            neuron_weight,//: HashMap::new(),
            //weight_index: 0,
            //next_weight_index: 1,
        }
    }

    //updates all the neurons 
    pub fn tick(&mut self) {

        //update each neuron
        for (nkey, neuron) in self.neuron_info.iter() {

            //check if current neuron weight surpasses its threshhold. If it does, "fire" the neuron and update child weights
            let n_weight = self.neuron_weight.get_mut(nkey).unwrap();
            if n_weight.refrence_weight > neuron.threshold {

                //reset weight because we fired
                n_weight.refrence_weight = 0;

                //for each child
                for connection in &neuron.outputs {
                    //update weight
                    if let Some(child_weight) = self.neuron_weight.get_mut(&connection.key) {

                        let old_weight = child_weight.new_weight;

                        //this variable represents the delta weight from last tick (it should have been reset to 0 at the end of last tick)
                        child_weight.new_weight += connection.weight;
                        child_weight.time = 0; //reset fire timeout



                        //debugging giant weights
                        {
                            if connection.weight.abs() > 37 {
                                println!("problem");
                            }

                            let mut apple = 0;

                            if child_weight.new_weight.abs() > 2000 {
                                apple = neuron.threshold + 3;
                            }

                            if connection.key.contains("MVR")
                            || connection.key.contains("MVL")
                            || connection.key.contains("MDL")
                            || connection.key.contains("MDR")
                            {
                                apple = neuron.threshold + 3;

                                //look for big jumps in weight ammount
                                if (old_weight - child_weight.new_weight).abs() > 2000 {
                                    apple += neuron.threshold + 3;
                                }

                            }
                            if apple > 0 {
                                let mut pear = 3;
                                pear += apple;
                            }
                        }
                    }
                }
            }
        }

        //update persistence timeout weight as needed, and make the next weight absolute
        for (_, weight) in self.neuron_weight.iter_mut() {

            //if neuron timed out and is set to time out, don't 
            if weight.time > weight.persistence && weight.persistence >= 0 {
                weight.time = 0; //reset fire timeout
                weight.refrence_weight = 0; //reset current weight
            } else {
                //test
                //let old_weight = weight.new_weight;

                //apply new accumulated weight to current weight
                weight.refrence_weight += weight.new_weight;
                
            }


            weight.new_weight = 0; //reset relative weight
            weight.time += 1; //increment timeout

        }



    }

    //returns True if the neuron was found and weight adjusted
    pub fn add_weight_to_neuron(&mut self, key: &String, weight: i32) -> bool {

        if let Some(child_weight) = self.neuron_weight.get_mut(key) {
            child_weight.new_weight += weight;
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
                    child_weight.new_weight += connection.weight;
                }
            }
            return true;
        }

        false
    }

    //if key exists, return Some(weight), optionally zeroing the weight that was read
    pub fn get_neuron_weight(&mut self, key: &String, zero: bool) -> Option<i32> {
        if let Some(child_weight) = self.neuron_weight.get_mut(key) {
            let output = child_weight.refrence_weight;
            if zero {
                child_weight.refrence_weight = 0;
            }
            return Some(output);
        } else {
            return None;
        }
    }

    pub fn get_neuron_weight_arr(&mut self, keys: &[String], zero: bool) -> i32 {
        let mut weight_sum = 0;

        for key in keys {
            weight_sum += self.get_neuron_weight(key, zero).unwrap_or(0);
        }
        weight_sum
    }

    pub fn get_weight_list(&self) -> &HashMap<String, NeuronWeight> {
        &self.neuron_weight
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
    pub fn stimulate_front_bump(&mut self) {
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

    //gets the sum of all weights at pre-defined muscle neurons
    pub fn get_mdl(&mut self) -> i32 {
        let key_aray = [
            format!("MDL01"),
            format!("MDL02"),
            format!("MDL03"),
            format!("MDL04"),
            format!("MDL05"),
            format!("MDL06"),
            format!("MDL07"),
            format!("MDL08"),
            format!("MDL09"),
            format!("MDL10"),
            format!("MDL11"),
            format!("MDL12"),
            format!("MDL13"),
            format!("MDL14"),
            format!("MDL15"),
            format!("MDL16"),
            format!("MDL17"),
            format!("MDL18"),
            format!("MDL19"),
            format!("MDL20"),
            format!("MDL21"),
            format!("MDL22"),
            format!("MDL23"),
            format!("MDL24"),
        ];
        self.get_neuron_weight_arr(&key_aray, true)
    }
    pub fn get_mvl(&mut self) -> i32 {
        let key_aray = [
            format!("MVL01"),
            format!("MVL02"),
            format!("MVL03"),
            format!("MVL04"),
            format!("MVL05"),
            format!("MVL06"),
            format!("MVL07"),
            format!("MVL08"),
            format!("MVL09"),
            format!("MVL10"),
            format!("MVL11"),
            format!("MVL12"),
            format!("MVL13"),
            format!("MVL14"),
            format!("MVL15"),
            format!("MVL16"),
            format!("MVL17"),
            format!("MVL18"),
            format!("MVL19"),
            format!("MVL20"),
            format!("MVL21"),
            format!("MVL22"),
            format!("MVL23"),
            format!("MVL24"),
        ];
        self.get_neuron_weight_arr(&key_aray, true)
    }
    pub fn get_mdr(&mut self) -> i32 {
        let key_aray = [
            format!("MDR01"),
            format!("MDR02"),
            format!("MDR03"),
            format!("MDR04"),
            format!("MDR05"),
            format!("MDR06"),
            format!("MDR07"),
            format!("MDR08"),
            format!("MDR09"),
            format!("MDR10"),
            format!("MDR11"),
            format!("MDR12"),
            format!("MDR13"),
            format!("MDR14"),
            format!("MDR15"),
            format!("MDR16"),
            format!("MDR17"),
            format!("MDR18"),
            format!("MDR19"),
            format!("MDR20"),
            format!("MDR21"),
            format!("MDR22"),
            format!("MDR23"),
            format!("MDR24"),
        ];
        self.get_neuron_weight_arr(&key_aray, true)
    }
    pub fn get_mvr(&mut self) -> i32 {
        let key_aray = [
            format!("MVR01"),
            format!("MVR02"),
            format!("MVR03"),
            format!("MVR04"),
            format!("MVR05"),
            format!("MVR06"),
            format!("MVR07"),
            format!("MVR08"),
            format!("MVR09"),
            format!("MVR10"),
            format!("MVR11"),
            format!("MVR12"),
            format!("MVR13"),
            format!("MVR14"),
            format!("MVR15"),
            format!("MVR16"),
            format!("MVR17"),
            format!("MVR18"),
            format!("MVR19"),
            format!("MVR20"),
            format!("MVR21"),
            format!("MVR22"),
            format!("MVR23"),
            format!("MVR24"),
        ];
        self.get_neuron_weight_arr(&key_aray, true)
    }



}





