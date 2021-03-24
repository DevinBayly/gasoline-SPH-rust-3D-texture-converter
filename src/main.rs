use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, Luma};
use std::fs::read;
use std::io::Cursor;
use std::thread;

#[derive(Debug, Clone, Copy)]
struct Particle {
    mass: f32,
    pos: [f32; 3],
    vel: [f32; 3],
    rho: f32,
    temp: f32,
    eps: f32,
    metals: f32,
    phi: f32,
}

struct Extent {
    min: f32,
    max: f32,
}

impl Extent {
    fn new(starting_value: f32) -> Self {
        Self {
            min: starting_value,
            max: starting_value,
        }
    }
    fn comp(&mut self, other: f32) {
        if other < self.min {
            self.min = other;
        } else if other > self.max {
            self.max = other;
        }
    }
    fn norm(&mut self, other: f32) -> f32 {
        (other - self.min) / (self.max - self.min)
    }
}

// want to create a way to see the actual positions
// still have
fn main() {
    let mut buff = read("./agora.000010").unwrap();
    let mut rdr = Cursor::new(buff);

    let time = rdr.read_f64::<LittleEndian>().unwrap();
    let nbodies = rdr.read_i32::<LittleEndian>().unwrap();
    let ndim = rdr.read_i32::<LittleEndian>().unwrap();
    let nsph = rdr.read_i32::<LittleEndian>().unwrap();
    let ndark = rdr.read_i32::<LittleEndian>().unwrap();
    let nstar = rdr.read_i32::<LittleEndian>().unwrap();
    let pad = rdr.read_i32::<LittleEndian>().unwrap();
    // first gas particle
    let mut sphs = vec![];
    for i in 0..nsph {
        sphs.push(Particle {
            mass: rdr.read_f32::<LittleEndian>().unwrap(),
            pos: [
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
            ],
            vel: [
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
            ],
            rho: rdr.read_f32::<LittleEndian>().unwrap(),
            temp: rdr.read_f32::<LittleEndian>().unwrap(),
            eps: rdr.read_f32::<LittleEndian>().unwrap(),
            metals: rdr.read_f32::<LittleEndian>().unwrap(),
            phi: rdr.read_f32::<LittleEndian>().unwrap(),
        });
    }
    println!("length of sphs {:?}", sphs.len());
    let mut x_extent = Extent::new(sphs[0].pos[0]);
    let mut y_extent = Extent::new(sphs[0].pos[1]);
    let mut z_extent = Extent::new(sphs[0].pos[2]);
    for particle in sphs.iter() {
        x_extent.comp(particle.pos[0]);
        y_extent.comp(particle.pos[1]);
        z_extent.comp(particle.pos[2]);
    }
    // go through and normalize the sphs
    let mut normalized_sphs = sphs.clone();
    for particle in normalized_sphs.iter_mut() {
        particle.pos[0] = x_extent.norm(particle.pos[0]);
        particle.pos[1] = y_extent.norm(particle.pos[1]);
        particle.pos[2] = z_extent.norm(particle.pos[2]);
    }
    // create an image
    let im_dims = (512 * 4, 512 * 4);
    let number_layers = 16 * 4;
    let layer_start = 0.57;
    let layer_end = 0.62;
    let thickness = (layer_end - layer_start) / (number_layers as f32);
    // amount of tiled images in the 3D texture
    let texture_side_number = (number_layers as f64).sqrt() as u32 + 1;
    println!(
        "num layers {} side number {}",
        number_layers, texture_side_number
    );
    let mut layers = vec![];
    for layer_index in 0..number_layers {
        println!("layer {} of {}", layer_index, number_layers);
        let mut layer_vec = vec![];
        for sph in normalized_sphs.iter() {
            if sph.pos[2] >= ((layer_index as f32) * thickness + layer_start)
                && sph.pos[2] < (((layer_index + 1) as f32) * thickness + layer_start)
            {
                layer_vec.push(sph.clone())
            }
        }
        println!(
            "slice {} {} has {} elements",
            ((layer_index as f32) * thickness + layer_start),
            (((layer_index + 1) as f32) * thickness + layer_start),
            layer_vec.len()
        );
        // run tile image maker on layer_vec
        layers.push(layer_vec);
    }
    //
    // for each image make a thread task to output it
    //let mut threads = vec![];
    //for layer_index in 0..layers.len() {
    //    let layer_vec = layers[layer_index].clone();
    //    threads.push(thread::spawn(move || {
    //        make_tile_image(layer_vec, layer_index as u32, im_dims, texture_side_number);
    //    }));
    //}
    //// call join on each
    //for thread in threads {
    //    thread.join().unwrap();
    //}

    // put single image together from the parts
    println!("assembling from tiles");
    // make the large image it will be imdims* texture_side_len x imdims*texture_side_len
    let mut three_d_texture: GrayImage = ImageBuffer::new(
        texture_side_number * im_dims.0,
        texture_side_number * im_dims.1,
    );
    println!("made big empty");
    let mut end = false;
    for row in 0..texture_side_number {
        if end {
            break;
        }
        for col in 0..texture_side_number {
            if end {
                break;
            }
            // these will be used to get the file name of an image
            println!("adding col {} row {}", col, row);
            match image::open(format!("test_points_{}_{}.png", row, col)) {
                Ok(im) => {
                    let generic_tile_img = im.into_luma8();
                    // copy into the 3D texture
                    three_d_texture
                        .copy_from(
                            &generic_tile_img,
                            col as u32 * im_dims.0,
                            row as u32 * im_dims.1,
                        )
                        .expect("copy fail");
                }
                _ => {
                    println!("reached end ");
                    end = true;
                    break;
                }
            };
        }
    }
    println!("saving");
    three_d_texture.save("3D_test.png").unwrap();
}
fn make_tile_image(
    layer_vector: Vec<Particle>,
    layer_index: u32,
    image_dims: (u32, u32),
    side_len: u32,
) {
    let mut img: GrayImage = ImageBuffer::new(image_dims.0, image_dims.1);

    // go through the normalized_particles and put them on the canvas if we find them by multiplying the xy by the dimensions

    for particle in layer_vector {
        // the -1 is to ensure we don't write at the far edge out of buffer
        img.put_pixel(
            (image_dims.0 as f32 * particle.pos[0] - 1.0).floor() as u32,
            (image_dims.1 as f32 * particle.pos[1] - 1.0).floor() as u32,
            Luma([255u8]),
        );
    }
    let name = format!(
        "test_points_{}_{}.png",
        (layer_index as f32 / side_len as f32) as u32,
        layer_index % side_len
    );

    // save the image out
    img.save(name.clone()).unwrap();
    println!("made image {}", name);
}
