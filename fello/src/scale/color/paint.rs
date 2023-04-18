use super::transform::Transform;
use read_fonts::tables::colr::*;
use read_fonts::ReadError;

/// Paint with flattened transforms.
pub enum PaintOrTransform<'a> {
    Paint(Paint<'a>),
    Transform(Transform, Paint<'a>),
}

pub fn flatten_transform<'a>(paint: Paint<'a>) -> Result<PaintOrTransform<'a>, ReadError> {
    match &paint {
        Paint::Transform(transform) => {
            let paint = transform.paint()?;
            let transform = transform.transform()?;
            Ok(PaintOrTransform::Transform(
                Transform {
                    xx: transform.xx().to_f64() as f32,
                    yx: transform.yx().to_f64() as f32,
                    xy: transform.xy().to_f64() as f32,
                    yy: transform.yy().to_f64() as f32,
                    dx: transform.dx().to_f64() as f32,
                    dy: transform.dy().to_f64() as f32,
                },
                paint,
            ))
        }
        Paint::VarTransform(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::Translate(transform) => Ok(PaintOrTransform::Transform(
            Transform::translate(
                transform.dx().to_i16() as f32,
                transform.dy().to_i16() as f32,
            ),
            transform.paint()?,
        )),
        Paint::VarTranslate(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::Rotate(transform) => Ok(PaintOrTransform::Transform(
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()),
            transform.paint()?,
        )),
        Paint::VarRotate(transform) => Ok(PaintOrTransform::Transform(
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()),
            transform.paint()?,
        )),
        Paint::RotateAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()).around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        )),
        Paint::VarRotateAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::Scale(transform) => Ok(PaintOrTransform::Transform(
            Transform::scale(transform.scale_x().to_f32(), transform.scale_y().to_f32()),
            transform.paint()?,
        )),
        Paint::VarScale(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::ScaleAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::scale(transform.scale_x().to_f32(), transform.scale_y().to_f32())
                .around_center(
                    transform.center_x().to_i16() as f32,
                    transform.center_y().to_i16() as f32,
                ),
            transform.paint()?,
        )),
        Paint::VarScaleAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::ScaleUniform(transform) => Ok(PaintOrTransform::Transform(
            Transform::scale(transform.scale().to_f32(), transform.scale().to_f32()),
            transform.paint()?,
        )),
        Paint::ScaleUniformAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::scale(transform.scale().to_f32(), transform.scale().to_f32()).around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        )),
        Paint::VarScaleUniform(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::VarScaleUniformAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::Skew(transform) => Ok(PaintOrTransform::Transform(
            Transform::skew(
                (transform.x_skew_angle().to_f32() * 180.0).to_radians(),
                (transform.y_skew_angle().to_f32() * 180.0).to_radians(),
            ),
            transform.paint()?,
        )),
        Paint::VarSkew(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        Paint::SkewAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::skew(
                (transform.x_skew_angle().to_f32() * 180.0).to_radians(),
                (transform.y_skew_angle().to_f32() * 180.0).to_radians(),
            )
            .around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        )),
        Paint::VarSkewAroundCenter(transform) => Ok(PaintOrTransform::Transform(
            Transform::default(),
            transform.paint()?,
        )),
        _ => Ok(PaintOrTransform::Paint(paint)),
    }
}
