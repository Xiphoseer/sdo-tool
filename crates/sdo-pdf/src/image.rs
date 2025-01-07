use pdf_create::{
    common::{ColorIs, ColorSpace, ImageMetadata},
    high::Image,
};
use signum::docs::{hcim::ImageSite, DocumentInfo};

/// Return a PDF Image for a site
pub(crate) fn image_for_site(di: &DocumentInfo, site: &ImageSite) -> Image {
    Image {
        meta: ImageMetadata {
            width: site.sel.w as usize,
            height: site.sel.h as usize,
            color_space: ColorSpace::DeviceGray,
            bits_per_component: 1,
            image_mask: true,
            decode: ColorIs::One,
        },
        data: di.image_at(site.img).select(site.sel),
    }
}
