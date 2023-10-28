use crate::*;

pub enum RenderTargetId {
    Named(String),
    Generated(u64),
}
