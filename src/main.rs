use brs;
use tobj;

use std::fs::File;

use image::{RgbaImage};
use cgmath::Vector4;

mod octree;
mod intersect;
mod barycentric;
mod voxelize;
mod color;
mod simplify;

use octree::VoxelTree;
use voxelize::voxelize;
use simplify::*;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "textured-voxelizer", about = "Voxelizes OBJ files to create textured voxel models")]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(parse(from_os_str))]
    output: PathBuf,

    #[structopt(long, possible_values = &["lossy", "lossless"], default_value = "lossy")]
    simplify: String,

    #[structopt(short, long, default_value = "1")]
    scale: f32
}

fn main() {
    let opt = Opt::from_args();
    let mut octree = VoxelTree::<Vector4::<u8>>::new();
    {
        match opt.file.extension() {
            Some(extension) => {
                match extension.to_str() {
                    Some("obj") => {}
                    _ => return println!("Only input files of type obj are supported")
                }
            },
            None => return println!("Invalid input file type")
        };

        let file = match opt.file.canonicalize() {
            Err(e) => return println!("Error encountered when looking for file {:?}: {:?}", opt.file, e.to_string()),
            Ok(f) => f
        };

        println!("Importing model...");
        let (mut models, materials) = tobj::load_obj(&file, true).unwrap();

        println!("Loading materials...");
        let mut material_images = Vec::<RgbaImage>::new();
        for material in materials {
            let image_path = file.canonicalize().unwrap().parent().unwrap().join(material.diffuse_texture);
            println!("{:?}", image_path);
            let image = image::open(image_path).unwrap().into_rgba();
            material_images.push(image);
        }

        println!("Voxelizing...");
        voxelize(&mut models, &material_images, &mut octree, opt.scale);
    }

    match opt.output.extension() {
        Some(extension) => {
            match extension.to_str() {
                Some("brs") => write_brs_data(&mut octree, opt.output, opt.simplify),
                Some(extension) => println!("Output file type {} is not supported", extension),
                None => println!("Invalid output file type")
            }
        },
        None => println!("Invalid output file type")
    }
}

fn write_brs_data(mut octree: &mut VoxelTree::<Vector4::<u8>>, output: PathBuf, simplify_algo: String) {
    let mut write_data = brs::Reader::new(File::open("blank.brs").unwrap()).unwrap().read_header1().unwrap().read_header2().unwrap().into_write_data().unwrap();
    write_data.bricks.clear();

    println!("Simplifying {:?}...", simplify_algo);
    if simplify_algo == "lossless" {
        simplify_lossless(&mut octree, &mut write_data);
    } else {
        simplify(&mut octree, &mut write_data);
    }

    // Write file
    println!("Writing file...");
    brs::write_save(&mut File::create(output).unwrap(), &write_data).unwrap();
}