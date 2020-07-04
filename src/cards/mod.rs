#[cfg(test)]
mod tests;

use super::guards::CardInRank;
use super::guards::CardOwner;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::guards::RankInBoard;
use super::models::*;
use super::persistence;
use super::persistence::Error;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};

#[post("/boards/<board_id>/ranks/<rank_id>/cards", data = "<post_card>")]
pub fn post_card(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  postgres: DatabaseConnection,
  board_id: String,
  rank_id: String,
  post_card: Json<PostCard>,
) -> Result<JsonValue, Status> {
  // Check that cards are open for the board
  let cards_open = match persistence::boards::cards_open(&postgres, &board_id, &participant_id.0) {
    Ok(b) => Ok(b),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(_) => Err(Status::InternalServerError),
  }?;

  if !cards_open {
    return Err(Status::Forbidden);
  }

  let new_card = NewCard {
    id: None,
    name: &post_card.name,
    description: &post_card.description,
    rank_id: &rank_id,
    participant_id: &participant_id.0,
    author: post_card.author.as_deref()
  };

  map_err!(
    persistence::cards::put_card(&postgres, new_card, &participant_id.0).map(|card| json!(card))
  )
}

#[get("/boards/<board_id>/cards")]
pub fn get_board_cards(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
  board_id: String,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::cards::get_board_cards(&postgres, &board_id, &participant_id.0)
      .map(|cards| json!(cards))
  )
}

#[get("/boards/<_board_id>/ranks/<rank_id>/cards")]
pub fn get_rank_cards(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  postgres: DatabaseConnection,
  _board_id: String,
  rank_id: String,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::cards::get_rank_cards(&postgres, &rank_id, &participant_id.0)
      .map(|cards| json!(cards))
  )
}

#[get("/boards/<_board_id>/ranks/<_rank_id>/cards/<card_id>")]
pub fn get_card(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  _board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  let card = map_err!(persistence::cards::get_card(
    &postgres,
    &card_id,
    &participant_id.0
  ))?;
  Ok(json!(card))
}

#[patch(
  "/boards/<board_id>/ranks/<rank_id>/cards/<card_id>",
  data = "<update_card>"
)]
#[allow(clippy::too_many_arguments)]
pub fn patch_card(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  _card_owner: CardOwner,
  postgres: DatabaseConnection,
  board_id: String,
  rank_id: String,
  card_id: String,
  update_card: Json<UpdateCard>,
) -> Result<JsonValue, Status> {
  // If a rank id was given and it's different to the current rank,
  // ensure it's still in the same board.
  if let Some(new_rank_id) = &update_card.rank_id {
    if *new_rank_id != rank_id {
      let new_rank_in_board =
        match persistence::ranks::rank_in_board(&postgres, new_rank_id, &board_id) {
          Ok(b) => Ok(b),
          Err(Error::NotFound) => Err(Status::NotFound),
          Err(_) => Err(Status::InternalServerError),
        }?;

      if !new_rank_in_board {
        return Err(Status::Forbidden);
      }
    }
  }

  map_err!(
    persistence::cards::patch_card(&postgres, &card_id, &update_card, &participant_id.0)
      .map(|card| json!(card))
  )
}

#[delete("/boards/<_board_id>/ranks/<_rank_id>/cards/<card_id>")]
pub fn delete_card(
  _participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  _board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<(), Status> {
  map_err!(persistence::cards::delete_card(&postgres, &card_id).map(|_| ()))
}
