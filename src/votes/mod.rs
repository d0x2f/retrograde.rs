use super::guards::BoardOwner;
use super::guards::CardInRank;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::guards::RankInBoard;
use super::models::*;
use super::persistence;
use diesel::result::Error;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::JsonValue;
use std::cmp::{max, min};

#[post("/boards/<board_id>/ranks/<_rank_id>/cards/<card_id>/vote")]
#[allow(clippy::too_many_arguments)]
pub fn post_vote(
  participant_id: ParticipantId,
  _board_owner: BoardOwner,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  let board = match persistence::get_board(&postgres, &board_id) {
    Ok(Some(b)) => Ok(b),
    Ok(None) => {
      error!("{}", "test");
      return Err(Status::NotFound);
    }
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("{}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  // Check that voting is open
  if !board.voting_open {
    return Err(Status::Forbidden);
  }

  // Get or create a new vote
  let new_vote = NewVote {
    card_id: &card_id,
    participant_id: &participant_id.0,
    count: Some(0),
  };

  let vote = persistence::put_vote(&postgres, &board_id, new_vote).map_err(|error| {
    error!("{}", error.to_string());
    Status::InternalServerError
  })?;

  // If max votes is already exceeded, abort the vote.
  // If the vote is greater than the max votes, it was probably
  // made when the limit was previously higher and so should stay.
  if vote.count >= board.max_votes {
    return Ok(json!(vote));
  }

  // Increment the vote
  let update_vote = UpdateVote {
    participant_id: &vote.participant_id,
    card_id: &vote.card_id,
    count: min(board.max_votes, vote.count + 1),
  };

  persistence::patch_vote(&postgres, &update_vote)
    .map(|v| json!(v))
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[delete("/boards/<board_id>/ranks/<_rank_id>/cards/<card_id>/vote")]
#[allow(clippy::too_many_arguments)]
pub fn delete_vote(
  participant_id: ParticipantId,
  _board_owner: BoardOwner,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  // Check that voting is open for the board
  let voting_open = match persistence::voting_open(&postgres, &board_id) {
    Ok(b) => Ok(b),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("{}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  if !voting_open {
    return Err(Status::Forbidden);
  }

  let vote = match persistence::get_vote(&postgres, &card_id, &participant_id.0) {
    Ok(Some(v)) => Ok(v),
    Ok(None) => Err(Status::NotFound),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("{}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  let update_vote = UpdateVote {
    participant_id: &vote.participant_id,
    card_id: &vote.card_id,
    count: max(0, vote.count - 1),
  };

  persistence::patch_vote(&postgres, &update_vote)
    .map(|v| json!(v))
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[get("/boards/<_board_id>/ranks/<_rank_id>/cards/<card_id>/votes")]
pub fn get_votes(
  _participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  _board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  let votes = persistence::get_votes(&postgres, &card_id).map_err(|error| {
    error!("{}", error.to_string());
    Status::InternalServerError
  })?;
  Ok(json!(votes))
}