use pdf_create::{
    common::{ColorIs, ColorSpace, ImageMetadata},
    high::{DictResource, Image, Res, XObject},
};
use signum::docs::{hcim::ImageSite, pbuf, DocumentInfo};

use crate::sdoc::Contents;

/// Return a PDF Image for a site
fn image_for_site(di: &DocumentInfo, site: &ImageSite) -> Image {
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

pub fn write_pdf_page_images(
    contents: &mut Contents,
    di: &DocumentInfo,
    page_info: &pbuf::Page,
    image_sites: &[ImageSite],
    res: &mut Res<'_>,
    x_objects: &mut DictResource<XObject>,
) -> bool {
    let mut has_images = false;
    for (index, site) in image_sites
        .iter()
        .enumerate()
        .filter(|(_, site)| site.page == page_info.phys_pnr)
    {
        let key = format!("I{}", index);
        log::debug!(
            "Adding image from #{} on page {} as /{}",
            site.img,
            page_info.log_pnr,
            &key
        );

        let image = image_for_site(di, site);

        contents.image(site, &key).unwrap();
        x_objects.insert(key.clone(), res.push_xobject(image));
        has_images |= true;
    }
    has_images
}
