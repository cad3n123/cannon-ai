#![allow(dead_code)]

use std::f32::consts::E;

use na::{self, DMatrix, DVector};
use rand::Rng;
use serde::{Deserialize, Serialize};
use typed_floats::Positive;

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    input_size: usize,
    output_size: usize,
    weights: Box<[DMatrix<f32>]>,
    biases: Box<[DVector<f32>]>,
}

impl NeuralNetwork {
    pub fn new_random_unchecked(layer_sizes: &[usize]) -> Self {
        let total_layers = layer_sizes.len();
        Self {
            input_size: layer_sizes[0],
            output_size: layer_sizes[total_layers - 1],
            weights: (1..total_layers)
                .map(|i| DMatrix::new_random(layer_sizes[i], layer_sizes[i - 1]))
                .collect::<Vec<DMatrix<f32>>>()
                .into_boxed_slice(),
            biases: (1..total_layers)
                .map(|i| DVector::new_random(layer_sizes[i]))
                .collect::<Vec<DVector<f32>>>()
                .into_boxed_slice(),
        }
    }
    pub fn new_random(layer_sizes: &[usize]) -> Result<Self, String> {
        let total_layers = layer_sizes.len();
        if total_layers <= 2 {
            return Err(
                "Neural network must have at least 2 layers for input and output.".to_string(),
            );
        }
        Ok(Self::new_random_unchecked(layer_sizes))
    }
    pub fn run_unchecked(&self, input: &DVector<f32>) -> DVector<f32> {
        let mut current_value = input.clone();
        for (weight, bias) in self.weights.iter().zip(self.biases.iter()) {
            current_value = weight * current_value + bias;
            current_value.apply(|value| *value = NeuralNetwork::activation_function(*value));
        }
        current_value
    }
    pub fn run(&self, input: &DVector<f32>) -> Result<DVector<f32>, String> {
        if input.nrows() != self.input_size {
            return Err(format!(
                "Incorrect input size for neural network. Expected {}",
                self.input_size
            ));
        }
        Ok(self.run_unchecked(input))
    }
    pub fn tweak_continuous(&mut self, change: Positive<f32>) {
        let mut rng = rand::thread_rng();
        let change_float: f32 = change.into();

        for weight in self.weights.iter_mut() {
            weight.apply(|element| {
                *element += rng.gen_range(-change_float..change_float) as f32;
                *element = element.clamp(-1.0, 1.0);
            });
        }
        for bias in self.biases.iter_mut() {
            bias.apply(|element| {
                *element += rng.gen_range(-change_float..change_float) as f32;
                *element = element.clamp(-1.0, 1.0);
            });
        }
    }
    pub fn tweak_discrete(&mut self, change: u32) {
        let mut rng = rand::thread_rng();
        let change_int = change as i32;

        for weight in self.weights.iter_mut() {
            weight.apply(|element| {
                *element += rng.gen_range(-change_int..change_int) as f32;
                *element = element.clamp(-1.0, 1.0);
            });
        }
        for bias in self.biases.iter_mut() {
            bias.apply(|element| {
                *element += rng.gen_range(-change_int..change_int) as f32;
                *element = element.clamp(-1.0, 1.0);
            });
        }
    }
    fn activation_function(value: f32) -> f32 {
        (E.powf(value)-E.powf(-value))/(E.powf(value)+E.powf(-value))
    }
}
