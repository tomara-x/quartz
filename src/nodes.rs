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

