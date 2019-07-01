use crate::filter::*;

#[test]
fn test_msresamp_rrrf_create_destroy() {
    let mut rs = MultiStageResampler::<f32>::new(2.0, 20.0);
    rs.reset();
    let mut dest = Vec::with_capacity(4);
    dest.resize(4, 0.0);
    rs.filter(&[1.0, 1.0], &mut dest).unwrap();
    assert!(dest[0] < 1.0);
}
