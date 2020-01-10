extern crate time;

use std::io::Write;

use self::time::{Duration, Timespec};

use crate::deserialize::{self, FromSql};
use crate::pg::{Pg, PgValue};
use crate::serialize::{self, Output, ToSql};
use crate::sql_types;

#[derive(FromSqlRow, AsExpression)]
#[diesel(foreign_derive)]
#[sql_type = "sql_types::Timestamp"]
#[allow(dead_code)]
struct TimespecProxy(Timespec);

const TIME_SEC_CONV: i64 = 946_684_800;

impl ToSql<sql_types::Timestamp, Pg> for Timespec {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let pg_epoch = Timespec::new(TIME_SEC_CONV, 0);
        let duration = *self - pg_epoch;
        let t = duration.num_microseconds().ok_or("Overflow error")?;
        ToSql::<sql_types::BigInt, Pg>::to_sql(&t, out)
    }
}

impl FromSql<sql_types::Timestamp, Pg> for Timespec {
    fn from_sql(bytes: Option<PgValue<'_>>) -> deserialize::Result<Self> {
        let t = <i64 as FromSql<sql_types::BigInt, Pg>>::from_sql(bytes)?;
        let pg_epoch = Timespec::new(TIME_SEC_CONV, 0);
        let duration = Duration::microseconds(t);
        let out = pg_epoch + duration;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use time::{Duration, Timespec};

    use crate::dsl::{now, sql};
    use crate::prelude::*;
    use crate::select;
    use crate::sql_types::Timestamp;
    use crate::test_helpers::*;

    fn connection() -> TestConnection {
        pg_connection()
    }

    #[test]
    fn unix_epoch_encodes_correctly() {
        let connection = connection();
        let query = select(sql::<Timestamp>("'1970-01-01'").eq(Timespec::new(0, 0)));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }

    #[test]
    fn unix_epoch_decodes_correctly() {
        let connection = connection();
        let epoch_from_sql =
            select(sql::<Timestamp>("'1970-01-01'::timestamp")).get_result::<Timespec>(&connection);
        assert_eq!(Ok(Timespec::new(0, 0)), epoch_from_sql);
    }

    #[test]
    fn times_relative_to_now_encode_correctly() {
        let connection = connection();
        let time = time::now_utc().to_timespec() + Duration::seconds(60);
        let query = select(now.at_time_zone("utc").lt(time));
        assert!(query.get_result::<bool>(&connection).unwrap());

        let time = time::now_utc().to_timespec() - Duration::seconds(60);
        let query = select(now.at_time_zone("utc").gt(time));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }
}
