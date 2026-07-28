use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    Context,
    args::BuildArgs,
    build::compile::compile_java,
    config::{ConfigProcesserDefinition, ProcesserType},
    lazy_java_error::LazyJavaError,
};

pub fn build_processors(build_args: &BuildArgs, ctx: &Context) -> Result<bool, LazyJavaError> {
    // TODO: add errors for this this is very lazy
    let mut processors = HashMap::new();
    if let Some(l) = ctx.config.processors() {
        for (class_name, processor) in l {
            processors.insert(class_name, processor.to_processer_definition()?);
        }
    }
    if processors.is_empty() {
        return Ok(true);
    }
    log::info!("Building {} annotation processor(s)", processors.len());

    let full_paths: Vec<PathBuf> = processors.iter().map(|(_k, p)| p.path.clone()).collect();

    if full_paths.is_empty() {
        return Ok(true);
    }

    log::debug!("Processor source paths: {:?}", full_paths);

    let _ = fs::remove_dir_all(&ctx.bin_processors.join("META-INF"));

    log::info!(
        "Compiling {} processor source(s) to {:?}",
        processors.len(),
        ctx.bin_processors
    );
    let result = compile_java(
        full_paths,
        &ctx.bin_processors,
        ctx,
        &build_args.javac_args,
        false,
    )?;
    log::info!(
        "Annotation processor compilation completed with status: {}",
        result
    );
    create_meta_folder(ctx, processors)?;

    Ok(true)
}

fn create_meta_folder(
    ctx: &Context,
    processors: HashMap<String, ConfigProcesserDefinition>,
) -> Result<(), LazyJavaError> {
    let dir = ctx.bin_processors.join("META-INF").join("services");

    let mut file_contents: String = String::new();
    let mut first = true;
    for (class_name, p) in processors
        .iter()
        .filter(|(_k, p)| p.kind == ProcesserType::Processor)
    {
        if !first {
            file_contents.push('\n');
        }
        file_contents.push_str(&format!("{}.{}", p.package, class_name));
        first = false;
    }

    log::debug!("Creating META-INF/services directory at {:?}", dir);
    fs::create_dir_all(&dir)?;

    let service_file = dir.join("javax.annotation.processing.Processor");
    let processor_count = processors
        .iter()
        .filter(|(_k, p)| p.kind == ProcesserType::Processor)
        .count();
    log::info!(
        "Writing processor service file to {:?} with {} entr{}",
        service_file,
        processor_count,
        if processor_count == 1 { "y" } else { "ies" }
    );
    log::debug!("Processor service file contents:\n{}", file_contents);
    fs::write(&service_file, file_contents)?;

    Ok(())
}
