use slint::ComponentHandle;

use crate::{database, global_update};
use crate::{App, GlobalDepartment};

pub async fn department(app: &App) -> anyhow::Result<()> {
    let myapp = app.clone_strong();
    app.global::<GlobalDepartment>().on_add_item(move |value| {
        let local_app = myapp.clone_strong();
        let _ = slint::spawn_local(async move {
            database::insert_department(value.to_string()).await.ok();
            global_update(&local_app).await.ok();
        });
    });
    let myapp = app.clone_strong();
    app.global::<GlobalDepartment>()
        .on_delete_item(move |value| {
            let local_app = myapp.clone_strong();
            let _ = slint::spawn_local(async move {
                database::delete_department(value.text.to_string())
                    .await
                    .ok();
                global_update(&local_app).await.ok();
            });
        });
    Ok(())
}
