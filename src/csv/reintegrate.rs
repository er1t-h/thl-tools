use std::{
    borrow::Cow,
    fs::{self, File},
    path::Path,
};

use csv::ReaderBuilder;

use crate::translate::{CSVStrategy, PatchError, Patcher};

pub fn reintegrate_csv(
    csv_file: &Path,
    original_mbe_file: Option<&Path>,
    destination: Option<&Path>,
) -> Result<(), PatchError<csv::Error>> {
    let should_remove_original;
    let original = if let Some(original) = original_mbe_file {
        Cow::Borrowed(original)
    } else {
        Cow::Owned(csv_file.with_extension("mbe"))
    };
    let (original_path, destination) = if let Some(target) = destination {
        should_remove_original = false;
        (original, Cow::Borrowed(target))
    } else {
        let tmp = original.with_extension("tmp");
        fs::rename(&original, &tmp)?;
        should_remove_original = true;
        (original, Cow::Owned(tmp))
    };

    let mut original = File::open(&original_path)?;
    let mut destination = File::create_new(&destination)?;
    let mut patcher = Patcher::new(&mut original, &mut destination);
    patcher.patch(CSVStrategy::new(
        ReaderBuilder::new()
            .from_path(csv_file)
            .map_err(PatchError::Strategy)?,
    ))?;
    if should_remove_original {
        fs::remove_file(original_path)?;
    }
    Ok(())
}
