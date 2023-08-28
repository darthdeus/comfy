use crate::*;

pub fn nine_patch_rect(
    rect: egui::Rect,
    cached_loader: &mut CachedImageLoader,
    egui_ctx: &egui::Context,
) -> egui::Shape {
    nine_patch_rect_ex(rect, cached_loader, egui_ctx, "panel-horizontal")
}

pub fn nine_patch_rect_ex(
    rect: egui::Rect,
    cached_loader: &mut CachedImageLoader,
    egui_ctx: &egui::Context,
    image: &str,
) -> egui::Shape {
    let size = rect.size();
    let top_left = rect.left_top();
    // let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
    // let top_left = response.rect.left_top();

    // let painter = ui.painter();

    let mut mesh = egui::Mesh::default();

    let corner = egui::vec2(64.0, 64.0);
    // let corner_uv = corner / size;
    let corner_uv = egui::vec2(0.49, 0.49);

    // TOP
    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(top_left + egui::vec2(0.0, 0.0), corner),
        egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(corner_uv.x, corner_uv.y),
        ),
        WHITE.egui(),
    );

    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(corner.x, 0.0),
            egui::vec2(size.x - 2.0 * corner.x, corner.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(corner_uv.x, 0.0),
            egui::pos2(1.0 - corner_uv.x, corner_uv.y),
        ),
        WHITE.egui(),
    );

    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(size.x - corner.x, 0.0),
            corner,
        ),
        egui::Rect::from_min_max(
            egui::pos2(1.0 - corner_uv.x, 0.0),
            egui::pos2(1.0, corner_uv.y),
        ),
        WHITE.egui(),
    );

    // MIDDLE
    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(0.0, corner.y),
            egui::vec2(corner.x, size.y - 2.0 * corner.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(0.0, corner_uv.y),
            egui::pos2(corner_uv.x, 1.0 - corner_uv.y),
        ),
        WHITE.egui(),
    );

    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(corner.x, corner.y),
            size - 2.0 * corner,
        ),
        egui::Rect::from_min_max(
            egui::pos2(corner_uv.x, corner_uv.y),
            egui::pos2(1.0 - corner_uv.x, 1.0 - corner_uv.y),
        ),
        WHITE.egui(),
    );

    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(size.x - corner.x, corner.y),
            egui::vec2(corner.x, size.y - 2.0 * corner.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(1.0 - corner_uv.x, corner_uv.y),
            egui::pos2(1.0, 1.0 - corner_uv.y),
        ),
        WHITE.egui(),
    );

    // BOTTOM
    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(0.0, size.y - corner.y),
            corner,
        ),
        egui::Rect::from_min_max(
            egui::pos2(0.0, 1.0 - corner_uv.y),
            egui::pos2(corner_uv.x, 1.0),
        ),
        WHITE.egui(),
    );

    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(corner.x, size.y - corner.y),
            egui::vec2(size.x - 2.0 * corner.x, corner.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(corner_uv.x, 1.0 - corner_uv.y),
            egui::pos2(corner_uv.x, 1.0),
        ),
        WHITE.egui(),
    );


    mesh.add_rect_with_uv(
        egui::Rect::from_min_size(
            top_left + egui::vec2(size.x - corner.x, size.y - corner.y),
            corner,
        ),
        egui::Rect::from_min_max(
            egui::pos2(1.0 - corner_uv.x, 1.0 - corner_uv.y),
            egui::pos2(1.0, 1.0),
        ),
        WHITE.egui(),
    );


    mesh.texture_id = cached_loader.image_or_err(egui_ctx, image);

    egui::Shape::mesh(mesh)
    // painter.add(egui::Shape::mesh(mesh));
}
