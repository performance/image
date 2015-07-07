//! Function for reading TIFF tags

use std::io::{self, Read, Seek};
use std::collections::{HashMap};

use super::stream::{ByteOrder, SmartReader, EndianReader};

use self::Value::{Unsigned, List};

macro_rules! tags {
    {$(
        $tag:ident
        $val:expr;
    )*} => {

        /// TIFF tag
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
        pub enum Tag {
            $($tag,)*
            Unknown(u16)
        }
        impl Tag {
            pub fn from_u16(n: u16) -> Tag {
                $(if n == $val { Tag::$tag } else)* {
                    Tag::Unknown(n)
                }
            }
        }
    }
}

// taken from https://partners.adobe.com/public/developer/en/tiff/TIFF6.pdf Appendix A
// TagName  Value; //  in HEx  TagTYPE    Number of Values               
tags! {
    NewSubfileType               254; //   FE LONG    1               
    SubfileType                  255; //   FF SHORT   1               
    ImageWidth                   256; //  100 SHORT   or  LONG    1       
    ImageLength                  257; //  101 SHORT   or  LONG    1       
    BitsPerSample                258; //  102 SHORT   SamplesPerPixel             
    Compression                  259; //  103 SHORT   1               
    PhotometricInterpretation    262; //  106 SHORT                   
    Threshholding                263; //  107 SHORT   1               
    CellWidth                    264; //  108 SHORT   1               
    CellLength                   265; //  109 SHORT   1               
    FillOrder                    266; //  10A SHORT   1               
    DocumentName                 269; //  10D ASCII                   
    ImageDescription             270; //  10E ASCII                   
    Make                         271; //  10F ASCII                   
    Model                        272; //  110 ASCII                   
    StripOffsets                 273; //  111 SHORT   or  LONG    StripsPerImage      
    Orientation                  274; //  112 SHORT   1               
    SamplesPerPixel              277; //  115 SHORT   1               
    RowsPerStrip                 278; //  116 SHORT   or  LONG    1       
    StripByteCounts              279; //  117 LONG    or  SHORT   StripsPerImage      
    MinSampleValue               280; //  118 SHORT   SamplesPerPixel             
    MaxSampleValue               281; //  119 SHORT   SamplesPerPixel             
    XResolution                  282; //  11A RATIONAL    1               
    YResolution                  283; //  11B RATIONAL    1               
    PlanarConfiguration          284; //  11C SHORT   1               
    PageName                     285; //  11D ASCII                   
    XPosition                    286; //  11E RATIONAL                    
    YPosition                    287; //  11F RATIONAL                    
    FreeOffsets                  288; //  120 LONG                    
    FreeByteCounts               289; //  121 LONG                    
    GrayResponseUnit             290; //  122 SHORT                   
    GrayResponseCurve            291; //  123 SHORT   2**BitsPerSample                
    T4Options                    292; //  124 LONG    1               
    T6Options                    293; //  125 LONG    1               
    ResolutionUnit               296; //  128 SHORT   1               
    PageNumber                   297; //  129 SHORT   2               
    TransferFunction             301; //  12D SHORT                   
    Software                     305; //  131 ASCII                   
    DateTime                     306; //  132 ASCII   20              
    Artist                       315; //  13B ASCII                   
    HostComputer                 316; //  13C ASCII                   
    Predictor                    317; //  13D SHORT   1               
    WhitePoint                   318; //  13E RATIONAL    2               
    PrimaryChromaticities        319; //  13F RATIONAL    6               
    ColorMap                     320; //  140 SHORT   3   *   (2**BitsPerSample)      
    HalftoneHints                321; //  141 SHORT   2               
    TileWidth                    322; //  142 SHORT   or  LONG    1       
    TileLength                   323; //  143 SHORT   or  LONG    1       
    TileOffsets                  324; //  144 LONG    TilesPerImage               
    TileByteCounts               325; //  145 SHORT   or  LONG    TilesPerImage       
    InkSet                       332; //  14C SHORT   1               
    InkNames                     333; //  14D ASCII   t               
    NumberOfInks                 334; //  14E SHORT   1               
    DotRange                     336; //  150 BYTE    or  SHORT   2,  or  2*
    TargetPrinter                337; //  151 ASCII   any             
    ExtraSamples                 338; //  152 BYTE    number  of  extra   compo   
    SampleFormat                 339; //  153 SHORT   SamplesPerPixel             
    SMinSampleValue              340; //  154 Any SamplesPerPixel             
    SMaxSampleValue              341; //  155 Any SamplesPerPixel             
    TransferRange                342; //  156 SHORT   6               
    JPEGProc                     512; //  200 SHORT   1               
    JPEGInterchangeFormat        513; //  201 LONG    1               
    JPEGInterchangeFormatLngth   514; //  202 LONG    1               
    JPEGRestartInterval          515; //  203 SHORT   1               
    JPEGLosslessPredictors       517; //  205 SHORT   SamplesPerPixel             
    JPEGPointTransforms          518; //  206 SHORT   SamplesPerPixel             
    JPEGQTables                  519; //  207 LONG    SamplesPerPixel             
    JPEGDCTables                 520; //  208 LONG    SamplesPerPixel             
    JPEGACTables                 521; //  209 LONG    SamplesPerPixel             
    YCbCrCoefficients            529; //  211 RATIONAL    3               
    YCbCrSubSampling             530; //  212 SHORT   2               
    YCbCrPositioning             531; //  213 SHORT   1               
    ReferenceBlackWhite          532; //  214 LONG    2*SamplesPerPixel               
    Copyright                  33432; // 8298    ASCII   Any             
}



// Note: These tags appear in the order they are mentioned in the TIFF reference
// https://partners.adobe.com/public/developer/en/tiff/TIFF6.pdf
// tags!{
//     // Baseline tags:
//     Artist 315; // TODO add support
//     // grayscale images PhotometricInterpretation 1 or 3
//     BitsPerSample 258;
//     CellLength 265; // TODO add support
//     CellWidth 264; // TODO add support
//     // palette-color images (PhotometricInterpretation 3)
//     ColorMap 320; // TODO add support
//     Compression 259; // TODO add support for 2 and 32773
//     Copyright 33432; // TODO add support
//     DateTime 306; // TODO add support
//     ExtraSamples 338; // TODO add support
//     FillOrder 266; // TODO add support
//     FreeByteCounts 289; // TODO add support
//     FreeOffsets 288; // TODO add support
//     GrayResponseCurve 291; // TODO add support
//     GrayResponseUnit 290; // TODO add support
//     HostComputer 316; // TODO add support
//     ImageDescription 270; // TODO add support
//     ImageLength 257;
//     ImageWidth 256;
//     Make 271; // TODO add support
//     MaxSampleValue 281; // TODO add support
//     MinSampleValue 280; // TODO add support
//     Model 272; // TODO add support
//     NewSubfileType 254; // TODO add support
//     Orientation 274; // TODO add support
//     PhotometricInterpretation 262;
//     PlanarConfiguration 284;
//     ResolutionUnit 296; // TODO add support
//     RowsPerStrip 278;
//     SamplesPerPixel 277;
//     Software 305;
//     StripByteCounts 279;
//     StripOffsets 273;
//     SubfileType 255; // TODO add support
//     Threshholding 263; // TODO add support
//     XResolution 282;
//     YResolution 283;
//     // Advanced tags
//     Predictor 317;
//     // TIFF Extensions
//     // Section 11 CCITT Bilevel Encodings
//     // Compression
//     T4Options 292;
//     T6Options 293;
//     // Section 12 Document Storagte and Retrieval
//     DocumentName 269;
//     PageName 285;
//     PageNumber 297;
//     XPosition 286;
//     YPosition 287;
//     // Section 13: LZW Compression
//     // Section 14: Differencing Predictor
    
//     // Section 15: Tiled Images -- Do not use both striporiented and tile-oriented fields in the same TIFF file
//     TileWidth 322;
//     TileLength 323;
//     TileOffsets 324;
//     TileByteCounts 325;
//     // Section 16: CMYK Images
//     InkSet 332;
//     NumberOfInks 334;
//     InkNames 333;
//     DotRange 336;
//     TargetPrinter 337;

//     // Section 17: HalftoneHints
//     HalftoneHints 321;
//     // Section 18: Associated Alpha Handling
//     ExtraSamples 338;
//     // Section 19: Data Sample Format    
//     SampleFormat 339;
//     SMinSampleValue 340;
//     SMaxSampleValue 341;
//     // Section 20: RGB Image Colorimetry
//     WhitePoint 318;
//     PrimaryChromaticities 319;
//     TransferFunction 301;
//     TransferRange 342;
//     ReferenceBlackWhite 532;
//     // Section 21: YCbCr    Images
// }

enum_from_primitive! {
#[derive(Clone, Copy, Debug)]
pub enum Type {
    BYTE      =  1,
    ASCII     =  2,
    SHORT     =  3,
    LONG      =  4,
    RATIONAL  =  5,
    SBYTE     =  6,
    UNDEFINED =  7,
    SSHORT    =  8,
    SLONG     =  9,
    SRATIONAL = 10,
    FLOAT     = 11,
    DOUBLE    = 12,
}

}


#[allow(unused_qualifications)]
#[derive(Debug)]
pub enum Value {
    //Signed(i32),
    Unsigned(u32),
    List(Vec<Value>)
}

#[allow(unused_qualifications)]
#[derive(Debug)]
pub enum Value_Type {
    Value,
    Offset
}


impl Value {
    pub fn as_u32(self) -> ::image::ImageResult<u32> {
        match self {
            Unsigned(val) => Ok(val),
            val => Err(::image::ImageError::FormatError(format!(
                "Expected unsigned integer, {:?} found.", val
            )))
        }
    }
    pub fn as_u32_vec(self) -> ::image::ImageResult<Vec<u32>> {
        match self {
            List(vec) => {
                let mut new_vec = Vec::with_capacity(vec.len());
                for v in vec.into_iter() {
                    new_vec.push(try!(v.as_u32()))
                }
                Ok(new_vec)
            },
            Unsigned(val) => Ok(vec![val]),
            //_ => Err(::image::FormatError("Tag data malformed.".to_string()))
        }
    }
}

pub struct Entry {
    type_: Type,
    count: u32,
    offset: [u8; 4]
}

impl ::std::fmt::Debug for Entry {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        fmt.write_str(&format!("Entry {{ type_: {:?}, count: {:?}, offset: {:?} }}",
            self.type_,
            self.count,
            &self.offset,
            // String::from_utf8_lossy ( &self.offset ),
        ))
    }
}

impl Entry {
    pub fn new(type_: Type, count: u32, offset: [u8; 4] ) -> Entry {
        Entry {
            type_: type_,
            count: count,
            offset: offset,
        }
    }

    /// Returns a mem_reader for the offset/value field
    pub fn r(&self, byte_order: ByteOrder) -> SmartReader<io::Cursor<Vec<u8>>> {
        SmartReader::wrap(
            io::Cursor::new(self.offset.to_vec()),
            byte_order
        )
    }
    // Refactor this to remove the dependency on decoder, 
    pub fn val<R: Read + Seek>(&self, decoder: &mut super::TIFFDecoder<R>)
    -> ::image::ImageResult<Value> {
        let bo = decoder.byte_order();
        match (self.type_, self.count) {
            // TODO check if this could give wrong results
            // at a different endianess of file/computer.
            (Type::BYTE, 1) => Ok(Unsigned(self.offset[0] as u32)),
            (Type::SHORT, 1) => Ok(Unsigned(try!(self.r(bo).read_u16()) as u32)),
            (Type::SHORT, 2) => {
                let mut r = self.r(bo);
                Ok(List(vec![
                    Unsigned(try!(r.read_u16()) as u32),
                    Unsigned(try!(r.read_u16()) as u32)
                ]))
            },
            (Type::SHORT, n) => {
                let mut v = Vec::with_capacity(n as usize);
                try!(decoder.goto_offset(try!(self.r(bo).read_u32())));
                for _ in 0 .. n {
                    v.push(Unsigned(try!(decoder.read_short()) as u32))
                }
                Ok(List(v))
            },
            (Type::LONG, 1) => Ok(Unsigned(try!(self.r(bo).read_u32()))),
            (Type::LONG, n) => {
                let mut v = Vec::with_capacity(n as usize);
                try!(decoder.goto_offset(try!(self.r(bo).read_u32())));
                for _ in 0 .. n {
                    v.push(Unsigned(try!(decoder.read_long())))
                }
                Ok(List(v))
            }
            _ => Err(::image::ImageError::UnsupportedError("Unsupported data type.".to_string()))
        }
    }
}

/// Type representing an Image File Directory
pub type Directory = HashMap<Tag, Entry>;
