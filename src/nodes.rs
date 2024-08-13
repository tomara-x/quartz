use crossbeam_channel::{Receiver, Sender};
use fundsp::fft::*;
use fundsp::hacker32::*;
use std::collections::VecDeque;

/// switch between nets based on index
/// - input 0: index
/// - output 0: output from selected net
#[derive(Default, Clone)]
pub struct Select {
    nets: Vec<Net>,
}

impl Select {
    /// create a select node. takes an array of nets
    pub fn new(nets: Vec<Net>) -> Self {
        Select { nets }
    }
}

impl AudioNode for Select {
    const ID: u64 = 1213;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
                (input[3] * self.sr).round() as usize,
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
}

/// index an array of floats
/// - input 0: index
/// - output 0: value at index
#[derive(Clone)]
pub struct ArrGet {
    arr: Vec<f32>,
}

impl ArrGet {
    pub fn new(arr: Vec<f32>) -> Self {
        ArrGet { arr }
    }
}

impl AudioNode for ArrGet {
    const ID: u64 = 1312;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut buffer = [0.];
        if let Some(n) = self.arr.get(input[0] as usize) {
            buffer[0] = *n;
        }
        buffer.into()
    }
}

/// shift register
/// - input 0: input signal
/// - input 1: trigger
/// - output 0...8: output from each index
#[derive(Default, Clone)]
pub struct ShiftReg {
    reg: [f32; 8],
}

impl ShiftReg {
    pub fn new() -> Self {
        ShiftReg { reg: [0.; 8] }
    }
}

impl AudioNode for ShiftReg {
    const ID: u64 = 1110;
    type Inputs = U2;
    type Outputs = U8;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[1] != 0. {
            self.reg[7] = self.reg[6];
            self.reg[6] = self.reg[5];
            self.reg[5] = self.reg[4];
            self.reg[4] = self.reg[3];
            self.reg[3] = self.reg[2];
            self.reg[2] = self.reg[1];
            self.reg[1] = self.reg[0];
            self.reg[0] = input[0];
        }
        self.reg.into()
    }

    fn reset(&mut self) {
        self.reg = [0., 0., 0., 0., 0., 0., 0., 0.];
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
    pub fn new(arr: Vec<f32>, range: f32) -> Self {
        Quantizer { arr, range }
    }
}

impl AudioNode for Quantizer {
    const ID: u64 = 1111;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
/// - inputs 0..: inputs to the net
/// - outputs 0..: last outputs from the net
#[derive(Clone)]
pub struct Kr {
    x: Net,
    n: usize,
    vals: Vec<f32>,
    count: usize,
    inputs: usize,
    outputs: usize,
    // set the sr of the inner net to sr/n to keep durations and frequencies unchanged
    preserve_time: bool,
}

impl Kr {
    pub fn new(x: Net, n: usize, preserve_time: bool) -> Self {
        let inputs = x.inputs();
        let outputs = x.outputs();
        let mut vals = Vec::new();
        vals.resize(outputs, 0.);
        Kr { x, n, vals, count: 0, inputs, outputs, preserve_time }
    }
}

impl AudioUnit for Kr {
    fn reset(&mut self) {
        self.x.reset();
        self.count = 0;
        self.vals.fill(0.);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.preserve_time {
            self.x.set_sample_rate(sample_rate / self.n as f64);
        } else {
            self.x.set_sample_rate(sample_rate);
        }
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        if self.count == 0 {
            self.count = self.n;
            self.x.tick(input, &mut self.vals);
        }
        self.count -= 1;
        output[..self.outputs].copy_from_slice(&self.vals[..self.outputs]);
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut i = 0;
        while i < size {
            if self.count == 0 {
                self.count = self.n;
                let mut tmp = Vec::new();
                for c in 0..input.channels() {
                    tmp.push(input.at_f32(c, i));
                }
                self.x.tick(&tmp, &mut self.vals);
            }
            self.count -= 1;
            for c in 0..output.channels() {
                output.set_f32(c, i, self.vals[c]);
            }
            i += 1;
        }
    }

    fn inputs(&self) -> usize {
        self.inputs
    }

    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 1112;
        ID
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(self.get_id()))
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn allocate(&mut self) {
        self.x.allocate();
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
        Reset { net, dur: s, n: (s * 44100.).round() as usize, count: 0 }
    }
}

impl AudioNode for Reset {
    const ID: u64 = 1113;
    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
}

/// reset network when triggered
/// - input 0: reset the net when non-zero
/// - output 0: output from the net
#[derive(Default, Clone)]
pub struct TrigReset {
    net: Net,
}

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
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let buffer = [self.val];
        self.val += input[0] / self.sr;
        if self.val >= 1. {
            self.val -= 1.;
        }
        buffer.into()
    }

    fn reset(&mut self) {
        self.val = 0.;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate as f32;
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
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let l = self.lr.try_recv().unwrap_or(0.);
        let r = self.rr.try_recv().unwrap_or(0.);
        [l, r].into()
    }
}

/// unit for swapping nodes
#[derive(Clone)]
pub struct SwapUnit {
    x: Net,
    receiver: Receiver<Net>,
    inputs: usize,
    outputs: usize,
}

impl SwapUnit {
    pub fn new(x: Net, receiver: Receiver<Net>) -> Self {
        let inputs = x.inputs();
        let outputs = x.outputs();
        Self { x, receiver, inputs, outputs }
    }
}

impl AudioUnit for SwapUnit {
    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        if let Ok(net) = self.receiver.try_recv() {
            if self.x.inputs() == net.inputs() && self.x.outputs() == net.outputs() {
                self.x = net;
            }
        }
        self.x.tick(input, output);
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if let Ok(net) = self.receiver.try_recv() {
            if self.x.inputs() == net.inputs() && self.x.outputs() == net.outputs() {
                self.x = net;
            }
        }
        self.x.process(size, input, output);
    }

    fn inputs(&self) -> usize {
        self.inputs
    }

    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 1118;
        ID
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(self.get_id()))
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}

/// rfft
/// - input 0: input
/// - output 0: real
/// - output 1: imaginary
#[derive(Default, Clone)]
pub struct Rfft {
    n: usize,
    input: Vec<f32>,
    output: Vec<Complex32>,
    count: usize,
    start: usize,
}

impl Rfft {
    pub fn new(n: usize, start: usize) -> Self {
        let mut input = Vec::new();
        let mut output = Vec::new();
        input.resize(n, 0.);
        output.resize(n / 2 + 1, Complex32::ZERO);
        Rfft { n, input, output, count: start, start }
    }
}

impl AudioNode for Rfft {
    const ID: u64 = 1120;
    type Inputs = U1;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let i = self.count;
        self.count += 1;
        if self.count == self.n {
            self.count = 0;
        }
        if i == 0 {
            real_fft(self.input.as_slice(), self.output.as_mut_slice());
        }
        self.input[i] = input[0];
        if i <= self.n / 2 {
            let out = self.output[i];
            [out.re, out.im].into()
        } else {
            let out = self.output[self.n - i].conj();
            [out.re, out.im].into()
        }
    }

    fn reset(&mut self) {
        self.count = self.start;
        self.input.fill(0.);
        self.output.fill(Complex32::ZERO);
    }
}

/// ifft
/// - input 0: real
/// - input 1: imaginary
/// - output 0: real
/// - output 1: imaginary
#[derive(Default, Clone)]
pub struct Ifft {
    n: usize,
    input: Vec<Complex32>,
    output: Vec<Complex32>,
    count: usize,
    start: usize,
}

impl Ifft {
    pub fn new(n: usize, start: usize) -> Self {
        let mut input = Vec::new();
        let mut output = Vec::new();
        input.resize(n, Complex32::ZERO);
        output.resize(n, Complex32::ZERO);
        Ifft { n, input, output, count: start, start }
    }
}

impl AudioNode for Ifft {
    const ID: u64 = 1121;
    type Inputs = U2;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let i = self.count;
        self.count += 1;
        if self.count == self.n {
            self.count = 0;
        }
        if i == 0 {
            inverse_fft(self.input.as_slice(), self.output.as_mut_slice());
        }
        self.input[i] = Complex32::new(input[0], input[1]);
        let buffer = [self.output[i].re, self.output[i].im];
        buffer.into()
    }

    fn reset(&mut self) {
        self.count = self.start;
        self.input.fill(Complex32::ZERO);
        self.output.fill(Complex32::ZERO);
    }
}

/// variable delay with input time in samples
/// - input 0: signal
/// - input 1: delay time in samples
/// - output 0: delayed signal
#[derive(Clone)]
pub struct SampDelay {
    buffer: VecDeque<f32>,
    max: usize,
}

impl SampDelay {
    pub fn new(max: usize) -> Self {
        let mut buffer = VecDeque::new();
        buffer.resize(max, 0.);
        SampDelay { buffer, max }
    }
}

impl AudioNode for SampDelay {
    const ID: u64 = 1122;
    type Inputs = U2;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.buffer.push_front(input[0]);
        let _ = self.buffer.pop_back();
        let out = self.buffer.get(input[1] as usize).unwrap_or(&0.);
        [*out].into()
    }

    fn reset(&mut self) {
        let mut new = VecDeque::new();
        new.resize(self.max, 0.);
        self.buffer = new;
    }
}

/// send samples to crossbeam channel
/// - input 0: input
/// - output 0: input passed through
#[derive(Clone)]
pub struct BuffIn {
    s: Sender<f32>,
}

impl BuffIn {
    pub fn new(s: Sender<f32>) -> Self {
        BuffIn { s }
    }
}

impl AudioNode for BuffIn {
    const ID: u64 = 1123;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let _ = self.s.try_send(input[0]);
        [input[0]].into()
    }
}

/// receive smaples from crossbeam channel
/// - output 0: output
#[derive(Clone)]
pub struct BuffOut {
    r: Receiver<f32>,
}

impl BuffOut {
    pub fn new(r: Receiver<f32>) -> Self {
        BuffOut { r }
    }
}

impl AudioNode for BuffOut {
    const ID: u64 = 1124;
    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [self.r.try_recv().unwrap_or(0.)].into()
    }
}

/// sample and hold
/// - input 0: input signal
/// - input 1: trigger
/// - output 0: held signal
#[derive(Clone)]
pub struct SnH {
    val: f32,
}

impl SnH {
    pub fn new() -> Self {
        SnH { val: 0. }
    }
}

impl AudioNode for SnH {
    const ID: u64 = 1125;
    type Inputs = U2;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if input[1] != 0. {
            self.val = input[0];
        }
        [self.val].into()
    }

    fn reset(&mut self) {
        self.val = 0.;
    }
}
