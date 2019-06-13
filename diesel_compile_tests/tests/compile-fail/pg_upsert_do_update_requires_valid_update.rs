#[macro_use] extern crate diesel;

use diesel::*;
use diesel::pg::upsert::*;

table! {
    users {
        id -> Integer,
        name -> VarChar,
    }
}

table! {
    posts {
        id -> Integer,
        title -> VarChar,
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser(#[column_name = "name"] &'static str);

#[allow(deprecated)]
fn main() {
    use self::users::dsl::*;
    let connection = PgConnection::establish("postgres://localhost").unwrap();

    // Valid update as sanity check
    insert_into(users).values(&NewUser("Sean")).on_conflict(id).do_update().set(name.eq("Sean")).execute(&connection);

    // No set clause
    insert_into(users)
        .values(&NewUser("Sean"))
        .on_conflict(id)
        .do_update()
        .execute(&connection);
        //~^ ERROR no method named `execute`

    // Update column from other table
    insert_into(users)
        .values(&NewUser("Sean"))
        .on_conflict(id)
        .do_update()
        .set(posts::title.eq("Sean"));
        //~^ ERROR E0271


    // Update column with value that is not selectable
    insert_into(users)
        .values(&NewUser("Sean"))
        .on_conflict(id)
        .do_update()
        .set(name.eq(posts::title));
        //~^ ERROR E0277

    // Update column with excluded value that is not selectable
    insert_into(users)
        .values(&NewUser("Sean"))
        .on_conflict(id)
        .do_update()
        .set(name.eq(excluded(posts::title)));
        //~^ ERROR type mismatch resolving `<posts::columns::title as diesel::Column>::Table == users::table`

    // Update column with excluded value of wrong type
    insert_into(users)
        .values(&NewUser("Sean"))
        .on_conflict(id)
        .do_update()
        .set(name.eq(excluded(id)));
        //~^ ERROR E0271

    // Excluded is only valid in upsert
    // FIXME: This should not compile
    update(users).set(name.eq(excluded(name))).execute(&connection);
}
