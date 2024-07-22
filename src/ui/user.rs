use std::rc::Rc;

use crate::database;
use slint::{ComponentHandle, ModelRc, SharedString, StandardListViewItem, VecModel};

use crate::{App, UserDetail, Users};
pub async fn get_user_list() -> anyhow::Result<Rc<VecModel<ModelRc<StandardListViewItem>>>> {
    let row_data = Rc::new(VecModel::default());
    let tmp = database::get_users().await?;
    for i in tmp {
        let items = Rc::new(VecModel::default());
        items.push(slint::format!("{0}", i.name.to_lowercase()).into());
        items.push(slint::format!("{}", i.login).into());
        items.push(slint::format!("{}", i.email).into());

        row_data.push(items.into());
    }
    Ok(row_data)
}

fn update_departments(app: &App) {
    let myapp = app.clone_strong();
    let _ = slint::spawn_local(async move {
        myapp
            .global::<UserDetail>()
            .set_departments(get_departs().await.unwrap_or_default());
    });
}

async fn get_departs() -> anyhow::Result<ModelRc<SharedString>> {
    let depart = database::get_department().await?;
    let mut row_data = Vec::default();
    for i in depart {
        let item = slint::format!("{}", i.name);
        row_data.push(item);
    }
    Ok(ModelRc::from(row_data.as_slice()))
}

pub async fn user_list(app: &App) -> anyhow::Result<()> {
    let row_data = get_user_list().await?;

    app.global::<Users>().set_row_data(row_data.clone().into());

    update_departments(app);
    Ok(())
}

pub async fn user_detail(app: &App) {
    let myapp = app.clone_strong();

    app.global::<UserDetail>().on_update(move || {
        let local_app = myapp.clone_strong();
        let _ = slint::spawn_local(async move { user_list(&local_app).await.unwrap() });
    });
    let myapp = app.clone_strong();
    app.global::<UserDetail>().on_create(move || {
        let local_app = myapp.clone_strong();
        let detail = myapp.global::<UserDetail>();
        let user = database::model::DbUser {
            name: detail.get_name().to_string(),
            login: detail.get_login().to_string(),
            email: detail.get_email().to_string(),
            department: detail.get_department().to_string(),
            document: detail.get_document().to_string(),
            id: 0,
            extension: detail.get_extension().to_string(),
            phone_number: detail.get_phone_number().to_string(),
        };
        let tmp = user.clone();
        let _ = slint::spawn_local(async move {
            let _ = database::create_user(tmp).await;
            let _ = user_list(&local_app).await;
        });
        detail.set_name(SharedString::default());
        detail.set_login(SharedString::default());
        detail.set_extension(SharedString::default());
        detail.set_email(SharedString::new());
        detail.set_department(SharedString::new());
        detail.set_document(SharedString::new());
        detail.set_id(0.to_string().into());
        detail.set_phone_number(SharedString::new());
    });

    let myapp = app.clone_strong();
    app.global::<Users>().on_select_user(move |user_login| {
        let local_app = myapp.clone_strong();
        let _ = slint::spawn_local(async move {
            let user_detail = local_app.global::<UserDetail>();
            let user = database::get_specific_user(user_login.to_string())
                .await
                .unwrap();
            let tmp = database::get_department_by_id(user.department.to_string())
                .await
                .unwrap();
            user_detail.set_name(user.name.into());
            user_detail.set_department(tmp.name.into());
            user_detail.set_document(user.document.into());
            user_detail.set_email(user.email.into());
            user_detail.set_extension(user.extension.into());
            user_detail.set_login(user.login.into());
            user_detail.set_phone_number(user.phone_number.into());
        });
    });

    let myapp = app.clone_strong();
    app.global::<UserDetail>().on_save(move || {
        let local_app = myapp.clone_strong();

        let _ = slint::spawn_local({
            let user_app = myapp.clone_strong();
            async move {
                let detail = user_app.global::<UserDetail>();
                let tmp = database::get_department_by_name(detail.get_department().to_string())
                    .await
                    .unwrap()
                    .id;
                let user = database::model::DbUser {
                    name: detail.get_name().to_string(),
                    login: detail.get_login().to_string(),
                    email: detail.get_email().to_string(),
                    id: tmp,
                    document: detail.get_document().to_string(),
                    department: detail.get_department().to_string(),
                    extension: detail.get_extension().to_string(),
                    phone_number: detail.get_phone_number().to_string(),
                };
                let tmp = user.clone();
                let _ = database::update_user(tmp).await;
                let _ = user_list(&local_app).await;
            }
        });
    });
}
