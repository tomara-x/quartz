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
        if input[0] != 0. {
            // remove existing events for that index
            self.events.retain(|&x| x.0 != input[1] as usize);
            // reset the net
            if let Some(network) = self.nets.get_mut(input[1] as usize) {
                network.reset();
            }
            // push the new event
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

/// shift register
/// - input 0: trigger
/// - input 1: input signal
/// - output 0...8: output from each index
#[derive(Default, Clone)]
pub struct ShiftReg {
    reg: [f32;8],
}

impl ShiftReg {
    pub fn new() -> Self { ShiftReg { reg: [0.;8] } }
}

impl AudioNode for ShiftReg {
    const ID: u64 = 1110;
    type Sample = f32;
    type Inputs = U2;
    type Outputs = U8;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if input[0] != 0. {
            self.reg[7] = self.reg[6];
            self.reg[6] = self.reg[5];
            self.reg[5] = self.reg[4];
            self.reg[4] = self.reg[3];
            self.reg[3] = self.reg[2];
            self.reg[2] = self.reg[1];
            self.reg[1] = self.reg[0];
            self.reg[0] = input[1];
        }
        self.reg.into()
    }
}

/// quantizer
/// - input 0: value to quantize
/// - output 0: quantized value
#[derive(Clone)]
pub struct Quantizer {
    arr: Vec<f32>,
    range: f32,
}

impl Quantizer {
    pub fn new(arr: Vec<f32>, range: f32) -> Self { Quantizer { arr, range } }
}

impl AudioNode for Quantizer {
    const ID: u64 = 1111;
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
        let n = input[0];
        let wrapped = n - self.range * (n / self.range).floor();
        let mut nearest = 0.;
        let mut dist = f32::MAX;
        for i in &self.arr {
            let d = (wrapped - i).abs();
            if d < dist {
                nearest = *i;
                dist = d;
            }
        }
        buffer[0] = n + nearest - wrapped;
        buffer.into()
    }
}


/// tick a network every n samples
/// - output 0: latest output from the net
#[derive(Default, Clone)]
pub struct Kr {
    net: Net32,
    n: usize,
    val: f32,
    count: usize,
}

impl Kr {
    pub fn new(net: Net32, n: usize) -> Self {
        Kr { net, n, val: 0., count: 0 }
    }
}

impl AudioNode for Kr {
    const ID: u64 = 1112;
    type Sample = f32;
    type Inputs = U0;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut buffer = [self.val];
        if self.count == 0 {
            self.count = self.n;
            self.net.tick(&[], &mut buffer);
            self.val = buffer[0];
        }
        self.count -= 1;
        buffer.into()
    }
}


/// reset network every s seconds
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct Reset {
    net: Net32,
    n: usize,
    count: usize,
}

impl Reset {
    pub fn new(net: Net32, s: f32) -> Self {
        Reset {
            net,
            n: (s * 44100.).round() as usize,
            count: 0,
        }
    }
}

impl AudioNode for Reset {
    const ID: u64 = 1113;
    type Sample = f32;
    type Inputs = U0;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut buffer = [0.];
        if self.count == self.n {
            self.net.reset();
            self.count = 0;
        }
        self.net.tick(&[], &mut buffer);
        self.count += 1;
        buffer.into()
    }
}

/// reset network when triggered
/// - input 0: reset the net when non-zero
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct TrigReset { net: Net32 }

impl TrigReset {
    pub fn new(net: Net32) -> Self {
        TrigReset { net }
    }
}

impl AudioNode for TrigReset {
    const ID: u64 = 1114;
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
        if input[0] != 0. {
            self.net.reset();
        }
        self.net.tick(&[], &mut buffer);
        buffer.into()
    }
}

/// reset network every s seconds (duration as input)
/// - input 0: reset interval
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct ResetV {
    net: Net32,
    count: usize,
}

impl ResetV {
    pub fn new(net: Net32) -> Self {
        ResetV {
            net,
            count: 0,
        }
    }
}

impl AudioNode for ResetV {
    const ID: u64 = 1115;
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
        if self.count >= (input[0] * 44100.).round() as usize {
            self.net.reset();
            self.count = 0;
        }
        self.net.tick(&[], &mut buffer);
        self.count += 1;
        buffer.into()
    }
}

/// phasor (ramp from 0..1)
/// - input 0: frequency
/// - output 0: ramp output
#[derive(Default, Clone)]
pub struct Ramp { val: f32 }

impl Ramp {
    pub fn new() -> Self {
        Ramp { val: 0. }
    }
}

impl AudioNode for Ramp {
    const ID: u64 = 1116;
    type Sample = f32;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let buffer = [self.val];
        self.val += input[0] / 44100.;
        if self.val >= 1. { self.val -= 1.; }
        buffer.into()
    }
}
