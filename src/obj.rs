use std::path::Path;
use std::io::{self, Read};
use std::fs::File;
use std::fmt;
use std::error;
use std::result;

use nalgebra::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: Vec3<f32>
}
implement_vertex!(Vertex, position);

impl From<Vec3<f32>> for Vertex {
    fn from(p: Vec3<f32>) -> Vertex {
        Vertex { position: p }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Normal {
    normal: Vec3<f32>
}
implement_vertex!(Normal, normal);

impl From<Vec3<f32>> for Normal {
    fn from(n: Vec3<f32>) -> Normal {
        Normal { normal: n }
    }
}

#[derive(Debug)]
pub struct Obj {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<u16>,
}

#[derive(Debug)]
pub enum ObjError {
    Io(io::Error),
    SyntaxError,
    NotSupported,
}

pub type Result<T> = result::Result<T, ObjError>;


pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Obj> {
    let mut file = try!(File::open(file_path));
    let mut contents = String::new();
    try!(file.read_to_string(&mut contents));
    parse(&contents)
}

fn parse(data: &str) -> Result<Obj> {
    let mut result = Obj {
        vertices: Vec::new(),
        normals: Vec::new(),
        indices: Vec::new(),
    };

    for line in data.lines() {
        if line.starts_with("v ") {
            result.vertices.push(Vertex::from(try!(parse_vec(line))))
        } else if line.starts_with("vn ") {
            result.normals.push(Normal::from(try!(parse_vec(line))))
        } else if line.starts_with("f ") {
            result.indices.extend(try!(parse_face(line)).iter())
        }
    }

    Ok(result)
}

fn parse_vec(line: &str) -> Result<Vec3<f32>> {
    let coords = try!(line.split_whitespace()
                          .skip(1)
                          .map(|s| s.parse::<f32>().map_err(|_| ObjError::SyntaxError))
                          .collect::<Result<Vec<_>>>());

    if coords.len() != 3 {
        return Err(ObjError::SyntaxError);
    }

    Ok(Vec3::new(coords[0], coords[1], coords[2]))
}

fn parse_face(line: &str) -> Result<Vec<u16>> {
    let verts = try!(line.split_whitespace()
             .skip(1)
             .map(parse_index)
             .collect::<Result<Vec<_>>>());

    if verts.len() != 3 {
        return Err(ObjError::SyntaxError);
    }

    Ok(verts)
}

fn parse_index(s: &str) -> Result<u16> {
    let inds = try!(s.split("//")
                     .map(|i| i.parse::<u16>()
                               .map(|i| i - 1)
                               .map_err(|_| ObjError::SyntaxError))
                     .collect::<Result<Vec<_>>>());
    if inds.len() != 2 {
        return Err(ObjError::SyntaxError);
    }
    if inds[0] != inds[1] {
        return Err(ObjError::NotSupported);
    }
    Ok(inds[0])
}


impl fmt::Display for ObjError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ObjError::Io(ref err) => write!(f, "IO error: {}", err),
            ObjError::SyntaxError => write!(f, "Syntax Error"),
            ObjError::NotSupported => write!(f, "Feature not supported"),
        }
    }
}

impl error::Error for ObjError {
    fn description(&self) -> &str {
        match *self {
            ObjError::Io(ref err) => err.description(),
            ObjError::SyntaxError => "Syntax Error",
            ObjError::NotSupported => "Feature not supported",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ObjError::Io(ref err) => Some(err),
            ObjError::SyntaxError => None,
            ObjError::NotSupported => None,
        }
    }
}

impl From<io::Error> for ObjError {
    fn from(err: io::Error) -> ObjError {
        ObjError::Io(err)
    }
}
