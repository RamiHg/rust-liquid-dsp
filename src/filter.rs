//! # Various FIR- and IIR-based filters.
//!
//! A wrapper on top of the filter module. Currently, only msresamp is implemented.

use std::convert::TryFrom as _;
use std::marker::PhantomData;
use std::os::raw::c_uint;
use std::os::raw::c_void;

use super::bindings::*;
use super::Error;

/// A wrapper on top of msresamp_tttf, a multi-stage arbitrary resampler. See the
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

    /// Calls msresamp_xxxf_print, which prints information about the resampler into stdout.
    pub fn print(&self) {
        unsafe {
            msresamp_rrrf_print(self.obj as *mut _);
        }
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

    /// Returns the approximate required capacity in a destination buffer passed to filter. The
    /// required capacity is calculated as `ceil(rate * input_length) + 1`. Note that any size that
    /// results in a unsigned 32-bit integer overflow - either before or after the rate change -
    /// is not compatible with liquid-dsp and will potentially corrupt memory. To protect against
    /// this, this function will panic if the length provided overflows a `u32`.
    pub fn needed_capacity(&self, input_length: usize) -> usize {
        debug_assert_le!(input_length, std::u32::MAX as usize);
        let needed_capacity = (input_length as f32 * self.rate()).ceil() + 1.0;
        assert_le!(needed_capacity, std::u32::MAX as f32);
        needed_capacity as usize
    }

    /// A wrapper around msresample_xxxf_execute that tries to retain some semblance of memory
    /// safety. Will return an `Error::CapacityError` if the given slice is not large enough for the
    /// expected number of samples. Returns the number of samples actually written. Will panic if
    /// the number of written samples exceeded `dest`'s capacity.
    pub fn filter(&mut self, data: &[f32], dest: &mut [f32]) -> Result<u32, Error> {
        // Ensure that the destination vector has enough capacity.
        let needed_capacity = self.needed_capacity(data.len());
        if dest.len() < needed_capacity {
            return Err(Error::CapacityError(needed_capacity, dest.len()));
        }
        let mut actual_written: c_uint = 0;
        unsafe {
            msresamp_rrrf_execute(
                self.obj as *mut _,
                data.as_ptr() as *mut _,
                c_uint::try_from(data.len()).unwrap(),
                dest.as_mut_ptr(),
                &mut actual_written,
            );
        }
        assert_le!(
            actual_written as usize,
            needed_capacity,
            "The actual number of bytes written exceeded the capacity of the destination slice."
        );
        Ok(actual_written)
    }
}

// A bit hacky - but otherwise would have to have separate structs for each type.
impl<T: 'static, InternalT> Drop for MultiStageResampler<T, InternalT> {
    fn drop(&mut self) {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            unsafe {
                msresamp_rrrf_destroy(self.obj as *mut _);
            }
        } else {
            unreachable!();
        }
    }
}
