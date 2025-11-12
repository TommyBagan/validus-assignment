use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(tonic_prost_build::compile_protos("proto/trade.proto")?)
}
