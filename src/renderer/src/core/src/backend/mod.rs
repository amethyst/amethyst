pub trait Resources: Copy + Clone + Eq + PartialEq {
    type Buffer: Copy + Clone + Eq + PartialEq;
    type Image: Copy + Clone + Eq + PartialEq;
    type Pipeline: Copy + Clone + Eq + PartialEq;
    type Sampler: Copy + Clone + Eq + PartialEq;
    type Target: Copy + Clone + Eq + PartialEq;
}
