//! # Fiters
//!
//! A wrapper on top of the filter module. Currently, only msresamp is implemented.

use std::convert::TryFrom as _;
use std::marker::PhantomData;
use std::os::raw::c_uint;
use std::os::raw::c_void;

use super::bindings::*;
use super::Error;

/// A wrapper on top of msresamp_tttf, a multi-stage arbitrary resampled. See the
/// [`official documentation`] for more details.
///
/// The types `T` and `InternalT` control the input/output and internal data types, respectively. For
/// example, `T=f32` and `InternalT=f32` correspond to the `msresamp_rrrf` family.
///
/// [`official documentation`]: https://liquidsdr.org/doc/msresamp/
pub struct MultiStageResampler<T: 'static, InternalT = T> {
    obj: *mut c_void,
    data_type: PhantomData<T>,
    inner_type: PhantomData<InternalT>,
}

impl MultiStageResampler<f32> {
    pub fn new(rate: f32, supression: f32) -> MultiStageResampler<f32> {
        let resampler = MultiStageResampler::<f32> {
            obj: unsafe { msresamp_rrrf_create(rate, supression) as *mut _ },
            data_type: PhantomData,
            inner_type: PhantomData,
        };
        assert_ne!(resampler.obj as usize, 0);
        resampler
    }

    pub fn reset(&mut self) {
        unsafe {
            msresamp_rrrf_reset(self.obj as *mut _);
        }
    }

    pub fn delay(&self) -> f32 {
        unsafe { msresamp_rrrf_get_delay(self.obj as *mut _) }
    }

    pub fn rate(&self) -> f32 {
        unsafe { msresamp_rrrf_get_rate(self.obj as *mut _) }
    }

    pub fn filter(&mut self, data: &[f32], dest: &mut [f32]) -> Result<(), Error> {
        // Ensure that the destination vector has enough capacity.
        let data_len = u32::try_from(data.len()).unwrap();
        let needed_capacity = (data_len as f32 * self.rate()).ceil();
        assert!(needed_capacity <= std::u32::MAX as f32);
        let needed_capacity = needed_capacity as usize;
        if dest.len() < needed_capacity {
            return Err(Error::CapacityError(needed_capacity, dest.len()));
        }
        let mut actual_written: c_uint = 0;
        unsafe {
            msresamp_rrrf_execute(
                self.obj as *mut _,
                data.as_ptr() as *mut _,
                data_len,
                dest.as_mut_ptr(),
                &mut actual_written,
            );
        }
        assert!(actual_written as usize <= needed_capacity);
        Ok(())
    }
}

// A bit hacky - but otherwise
impl<T: 'static, InternalT> Drop for MultiStageResampler<T, InternalT> {
    fn drop(&mut self) {
        match std::any::TypeId::of::<&'static T>() {
            _ => (),
        }
    }
}