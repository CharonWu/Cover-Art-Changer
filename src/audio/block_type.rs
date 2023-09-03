#[derive(Debug, PartialEq, Eq)]
pub enum FlacBlockType {
    StreamInfo,
    Padding,
    Application,
    Seekable,
    VorbisComment,
    CueSheet,
    Picture,
    Reserved,
    Invalid,
}
