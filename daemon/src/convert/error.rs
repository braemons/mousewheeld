// SPDX-License-Identifier: AGPL-3.0-or-later
//! One refusal shape, for every route.

use crate::model::error::ApiErrorBody;
use crate::wire;

pub fn error_to_wire(body: ApiErrorBody) -> wire::Error {
    wire::Error {
        error: body.error,
        detail: body.detail,
        context: body.context,
    }
}
