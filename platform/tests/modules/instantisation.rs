use platform::platform::platform_test::*;
use scrypto_test::prelude::*;
use std::path::Path;

#[test]
fn simple_package_can_be_published() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();

    // Act & Assert
    let _ = PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let initial_path = Path::new(this_package!())
        .join("../strategies/yield_multiplier/weft/")
        .canonicalize()
        .unwrap();
    let path = initial_path.to_str().unwrap();

    let _ = PackageFactory::compile_and_publish(path, &mut env, CompileProfile::Fast)?;

    Ok(())
}
