use crate::filter::*;

#[test]
fn test_msresamp_rrrf_create_destroy() {
    let mut rs = MultiStageResampler::<f32>::new(2.0, 20.0);
    rs.reset();
}

#[test]
fn test_msresamp_rrrf_filter() {
    let mut rs = MultiStageResampler::<f32>::new(2.0, 20.0);
    let mut dest = vec![0.0; 5];
    assert_eq!(rs.filter(&[1.0, 1.0], &mut dest), Ok(4));
}

#[test]
fn test_msresamp_rrrf_rate() {
    assert_eq!(MultiStageResampler::<f32>::new(2.0, 20.0).rate(), 2.0);
}

#[test]
fn test_msresamp_rrrf_needed_capacity() {
    assert_eq!(MultiStageResampler::<f32>::new(2.0, 20.0).needed_capacity(2), 5);
}