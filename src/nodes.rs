use fundsp::hacker32::*;

/// switch between nets based on index
/// - input 0: index
/// - output 0: output from selected net
#[derive(Default, Clone)]
pub struct Select {
    nets: Vec<Net32>,
}

impl Select {
    /// create a select node. takes an array of nets
    pub fn new(nets: Vec<Net32>) -> Self { Select {nets} }
}

impl AudioNode for Select {
    const ID: u64 = 1213;
    type Sample = f32;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut buffer = [0.];
        if let Some(network) = self.nets.get_mut(input[0] as usize) {
            network.tick(&[], &mut buffer);
        }
        buffer.into()
    }
}


/// sequence nets
/// - input 0: trigger
/// - input 1: index of network to play
/// - input 2: delay time
/// - input 3: duration
/// - output 0: output from all playing nets
#[derive(Default, Clone)]
pub struct Seq {
    nets: Vec<Net32>,
    // index, delay, duration (times in samples)
    events: Vec<(usize, usize, usize)>,
}

impl Seq {
    pub fn new(nets: Vec<Net32>) -> Self {
        Seq { nets, events: Vec::new() }
    }
}

impl AudioNode for Seq {
    const ID: u64 = 1729;
    type Sample = f32;
    type Inputs = U4;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // triggered, add an event
        if input[0] == 1. {
            self.events.push((
                input[1] as usize,
                (input[2] * 44100.).round() as usize,
                (input[3] * 44100.).round() as usize
            ));
        }
        // remove finished events
        self.events.retain(|&x| x.2 != 0);

        let mut buffer = [0.];
        let mut out = [0.];
        for i in &mut self.events {
            if i.1 == 0 {
                if let Some(network) = self.nets.get_mut(i.0) {
                    network.tick(&[], &mut buffer);
                    out[0] += buffer[0];
                    if i.2 == 1 { network.reset(); }
                }
                i.2 -= 1;
            } else {
                i.1 -= 1;
            }
        }
        out.into()
    }
}


/// index an array of floats
/// - input 0: index
/// - output 0: value at index
#[derive(Clone)]
pub struct ArrGet {
    arr: Vec<f32>,
}

impl ArrGet {
    pub fn new(arr: Vec<f32>) -> Self { ArrGet {arr} }
}

impl AudioNode for ArrGet {
    const ID: u64 = 1312;
    type Sample = f32;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut buffer = [0.];
        if let Some(n) = self.arr.get(input[0] as usize) {
            buffer[0] = *n;
        }
        buffer.into()
    }
}
