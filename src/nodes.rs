use fundsp::hacker32::*;
use crossbeam_channel::Receiver;

/// switch between nets based on index
/// - input 0: index
/// - output 0: output from selected net
#[derive(Default, Clone)]
pub struct Select {
    nets: Vec<Net>,
}

impl Select {
    /// create a select node. takes an array of nets
    pub fn new(nets: Vec<Net>) -> Self { Select {nets} }
}

impl AudioNode for Select {
    const ID: u64 = 1213;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if let Some(network) = self.nets.get_mut(input[0] as usize) {
            network.tick(&[], &mut buffer);
        }
        buffer.into()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        for net in &mut self.nets {
            net.set_sample_rate(sample_rate);
        }
    }

    fn reset(&mut self) {
        for net in &mut self.nets {
            net.reset();
        }
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
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
    nets: Vec<Net>,
    // index, delay, duration (times in samples)
    events: Vec<(usize, usize, usize)>,
    sr: f32,
}

impl Seq {
    pub fn new(nets: Vec<Net>) -> Self {
        Seq { nets, events: Vec::new(), sr: 44100. }
    }
}

impl AudioNode for Seq {
    const ID: u64 = 1729;
    type Inputs = U4;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
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
                (input[2] * self.sr).round() as usize,
                (input[3] * self.sr).round() as usize
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

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate as f32;
        for net in &mut self.nets {
            net.set_sample_rate(sample_rate);
        }
    }

    fn reset(&mut self) {
        for net in &mut self.nets {
            net.reset();
        }
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
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
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if let Some(n) = self.arr.get(input[0] as usize) {
            buffer[0] = *n;
        }
        buffer.into()
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
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
    type Inputs = U2;
    type Outputs = U8;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
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

    fn reset(&mut self) {
        self.reg = [0., 0., 0., 0., 0., 0., 0., 0.];
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
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
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
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

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}


/// tick a network every n samples
/// - output 0: latest output from the net
#[derive(Default, Clone)]
pub struct Kr {
    net: Net,
    n: usize,
    val: f32,
    count: usize,
}

impl Kr {
    pub fn new(net: Net, n: usize) -> Self {
        Kr { net, n, val: 0., count: 0 }
    }
}

impl AudioNode for Kr {
    const ID: u64 = 1112;
    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [self.val];
        if self.count == 0 {
            self.count = self.n;
            self.net.tick(&[], &mut buffer);
            self.val = buffer[0];
        }
        self.count -= 1;
        buffer.into()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.net.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.count = 0;
        self.val = 0.;
        self.net.reset();
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}


/// reset network every s seconds
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct Reset {
    net: Net,
    dur: f32,
    n: usize,
    count: usize,
}

impl Reset {
    pub fn new(net: Net, s: f32) -> Self {
        Reset {
            net,
            dur: s,
            n: (s * 44100.).round() as usize,
            count: 0,
        }
    }
}

impl AudioNode for Reset {
    const ID: u64 = 1113;
    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if self.count >= self.n {
            self.net.reset();
            self.count = 0;
        }
        self.net.tick(&[], &mut buffer);
        self.count += 1;
        buffer.into()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.n = (self.dur * sample_rate as f32).round() as usize;
        self.net.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.count = 0;
        self.net.reset();
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// reset network when triggered
/// - input 0: reset the net when non-zero
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct TrigReset { net: Net }

impl TrigReset {
    pub fn new(net: Net) -> Self {
        TrigReset { net }
    }
}

impl AudioNode for TrigReset {
    const ID: u64 = 1114;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if input[0] != 0. {
            self.net.reset();
        }
        self.net.tick(&[], &mut buffer);
        buffer.into()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.net.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.net.reset();
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// reset network every s seconds (duration as input)
/// - input 0: reset interval
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct ResetV {
    net: Net,
    count: usize,
    sr: f32,
}

impl ResetV {
    pub fn new(net: Net) -> Self {
        ResetV { net, count: 0, sr: 44100. }
    }
}

impl AudioNode for ResetV {
    const ID: u64 = 1115;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if self.count >= (input[0] * self.sr).round() as usize {
            self.net.reset();
            self.count = 0;
        }
        self.net.tick(&[], &mut buffer);
        self.count += 1;
        buffer.into()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate as f32;
        self.net.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.count = 0;
        self.net.reset();
    }

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// phasor (ramp from 0..1)
/// - input 0: frequency
/// - output 0: ramp output
#[derive(Default, Clone)]
pub struct Ramp {
    val: f32,
    sr: f32,
}

impl Ramp {
    pub fn new() -> Self {
        Ramp { val: 0., sr: 44100. }
    }
}

impl AudioNode for Ramp {
    const ID: u64 = 1116;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let buffer = [self.val];
        self.val += input[0] / self.sr;
        if self.val >= 1. { self.val -= 1.; }
        buffer.into()
    }

    fn reset(&mut self) {
        self.val = 0.;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate as f32;
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}


/// node that receives samples from crossbeam channels
/// - output 0: left
/// - output 1: right
#[derive(Clone)]
pub struct InputNode {
    lr: Receiver<f32>,
    rr: Receiver<f32>,
}

impl InputNode {
    pub fn new(lr: Receiver<f32>, rr: Receiver<f32>) -> Self {
        InputNode { lr, rr }
    }
}

impl AudioNode for InputNode {
    const ID: u64 = 1117;
    type Inputs = U0;
    type Outputs = U2;

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let l = self.lr.recv().unwrap_or(0.);
        let r = self.rr.recv().unwrap_or(0.);
        [l, r].into()
    }

    //fn reset(&mut self) {
    //}

    //fn set_sample_rate(&mut self, sample_rate: f64) {
    //}

    // TODO
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

