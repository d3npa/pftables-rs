use crate::bridge::*;
use std::error::Error;

#[test]
fn test_pfioc_table() -> Result<(), Box<dyn Error>> {
    let mut io = PfIocTable::new("my_table");
    io.buffer.push(PfrAddr::default());

    let mut io_c = io.translate()?;
    assert_eq!(io_c.pfrio_size, 1);

    // Simulate kernel interaction
    io_c.pfrio_size = 2;

    io.update(io_c)?;
    assert_eq!(io.size, 2);
    
    Ok(())
}