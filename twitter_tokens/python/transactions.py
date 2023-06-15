from solana.rpc.api import Client
import solana.system_program as sp
from solana.publickey import PublicKey
from solana.account import Account
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.rpc.types import TxOpts
import solana as sol
import spl.token.instructions as spl_token_instructions
from spl.token.constants import ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID

from borsh_construct import Enum, I32, CStruct, U8, U32, U64, HashMap, String
import base64
import numpy as np
import tweepy
import datetime
import json
from rpc_funcs import *

MINT_KEY = PublicKey("ESxUiMdmrZzJBk1JryJyVpD2ok9cheTV43H1HXEy8n5x")

VALID_HASHTAGS = ["DaoPlaysPokemon", "DaoPlaysRewards"]

IDMap = CStruct(
    "twitter_id" / U64,
    "error_code" / U8
)

RewardMark = CStruct(
    "mark" / U8
)

def load_key(filename):
	skey = open(filename).readlines()[0][1:-1].split(",")
	int_key = []
	for element in skey:
		int_key.append(int(element))
		
	owner=sol.keypair.Keypair.from_secret_key(bytes(int_key)[:32])
	
	return owner

def load_config(filename):
    
    return json.load(open(filename))["config"]


def get_followers():

    config = load_config("config.json")
    bearer_token = config["bearer_token"]

    client = tweepy.Client(bearer_token)

    # list followers
    followers = []
    for response in tweepy.Paginator(client.get_users_followers, 1532485814051012608,
                                        max_results=1000, limit=5):
        for f in response.data:
            followers.append(f.id)

    return followers

def check_if_user_is_following(twitter_id):

    followers = get_followers()

    

    if twitter_id in followers:
        return True

    return False

def process_tweet(tweet_id):

    config = load_config("config.json")
    bearer_token = config["bearer_token"]

    client = tweepy.Client(bearer_token)

    # Get Tweets

    # This endpoint/method returns a variety of information about the Tweet(s)
    # specified by the requested ID or list of IDs

    tweet_ids = [str(tweet_id)]

    # By default, only the ID and text fields of each Tweet will be returned
    # Additional fields can be retrieved using the tweet_fields parameter
    response = client.get_tweets(tweet_ids, tweet_fields=["created_at", "author_id", "entities"])

    tweet = response.data[0]
    entities = tweet.entities

    hashtags = []
    if (entities != None):
        if ("hashtags" in entities.keys()):
            for hash in entities['hashtags']:
                hashtags.append(hash['tag'])

    print(tweet.created_at.tzinfo == datetime.timezone.utc)
    print(tweet.created_at.date())
    print(tweet.text)
    print(tweet.author_id)

    return tweet.text, tweet.author_id, hashtags

def get_retweeters(tweet_id):

    config = load_config("config.json")

    bearer_token = config["bearer_token"]

    client = tweepy.Client(bearer_token)

    # getting the retweeters
    retweets_list = client.get_retweeters(tweet_id)

    retweeter_ids = []
    for rt in retweets_list.data:
        retweeter_ids.append(rt.id)

    return retweeter_ids
  
def get_user_id_map_data(dev_client, user_account_key):

    user_id_map_account, _id_map_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(program_key))

    response = dev_client.get_account_info(user_id_map_account)
    data = response["result"]["value"]["data"][0]
    decoded_data = base64.b64decode(data)

    id_map = IDMap.parse(decoded_data)
    twitter_id = id_map.twitter_id
    error_code = id_map.error_code

    return twitter_id, error_code

def get_reward_mark_data(dev_client, hashtag, tweet_id, user_id):

    user_id = np.uint64(user_id)
    tweet_id = np.uint64(tweet_id)

    user_hashtag_account, _hashtag_bump = PublicKey.find_program_address([bytes(hashtag, encoding="utf8"), tweet_id.tobytes(), user_id.tobytes()], PublicKey(program_key))

    response = dev_client.get_account_info(user_hashtag_account)
    data = response["result"]["value"]["data"][0]
    decoded_data = base64.b64decode(data)

    reward_mark_data = RewardMark.parse(decoded_data)
    reward_mark = reward_mark_data.mark

    return reward_mark


def get_set_user_id_idx(user_id,  user_account_key):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    user_id = np.uint64(user_id)
    user_id_map_account, _id_map_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(program_key))

    instruction = TransactionInstruction(
        program_id = program_key,
        data = Twitter_Instructions.build(Twitter_Instructions.enum.SetUserID(user_id = user_id)),
        keys = [
            AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),
            AccountMeta(pubkey=PublicKey(user_account_key), is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_id_map_account, is_signer=False, is_writable=True)        
            ]
    )

    return instruction


def get_set_error_idx(error_code, user_account_key):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    user_id_map_account, _id_map_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(program_key))

    idx = Twitter_Instructions.build(Twitter_Instructions.enum.SetError(error_code=error_code))

    instruction = TransactionInstruction(
        program_id = program_key,
        data = idx,
        keys = [
            AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),
            AccountMeta(pubkey=PublicKey(user_account_key), is_signer=False, is_writable=False),
            AccountMeta(pubkey=user_id_map_account, is_signer=False, is_writable=True)
        ]
    )

    return instruction

def set_user_error(dev_client, user_account_key, error_code):

    # (2) Create a new Keypair for the new account
    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    set_error_idx = get_set_error_idx(error_code, user_account_key)

    blockhash = dev_client.get_recent_blockhash()['result']['value']['blockhash']
    txn = Transaction(recent_blockhash=blockhash, fee_payer=wallet.public_key)
    txn.add(set_error_idx)
    txn.sign(wallet)

    response = dev_client.send_transaction(
        txn,
        wallet,
        opts=TxOpts(skip_preflight=True, skip_confirmation=False)
    )

    print(response)

def get_new_follower_idx(user_id, user_account_key):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    user_id = np.uint64(user_id)
    user_data_account, _data_bump = PublicKey.find_program_address([user_id.tobytes()], PublicKey(program_key))

    user_account = PublicKey(user_account_key)
    user_token_account = spl_token_instructions.get_associated_token_address(user_account, MINT_KEY)

    program_derived_account, _pda_bump = PublicKey.find_program_address([bytes("token_account", encoding="utf-8")], PublicKey(program_key))
    program_token_account = spl_token_instructions.get_associated_token_address(program_derived_account, MINT_KEY)

    idx = Twitter_Instructions.build(Twitter_Instructions.enum.NewFollower(user_id=user_id))

    instruction = TransactionInstruction(
        program_id = program_key,
        data = idx,
        keys = [
            AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),

            AccountMeta(pubkey=PublicKey(user_account_key), is_signer=False, is_writable=False),
            AccountMeta(pubkey=user_data_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_token_account, is_signer=False, is_writable=True),

            AccountMeta(pubkey=program_derived_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=program_token_account, is_signer=False, is_writable=True),

            AccountMeta(pubkey=MINT_KEY, is_signer=False, is_writable=False),

            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=sp.SYS_PROGRAM_ID, is_signer=False, is_writable=False)
        ]
    )

    return instruction

def new_follower(dev_client, user_id, user_account_key):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    new_follower_idx = get_new_follower_idx(user_id, user_account_key)

    blockhash = dev_client.get_recent_blockhash()['result']['value']['blockhash']
    txn = Transaction(recent_blockhash=blockhash, fee_payer=wallet.public_key)
    txn.add(new_follower_idx)
    txn.sign(wallet)

    response = dev_client.send_transaction(
        txn,
        wallet,
        opts=TxOpts(skip_preflight=True, skip_confirmation=False)
    )

    print(response)


def get_send_hashtag_reward_idx(user_account_key, user_id, tweet_id, hashtag):

    # This function expects to be passed three accounts, get them all first and then check their value is as expected

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    user_id_map_account, _id_map_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(program_key))

    user_id = np.uint64(user_id)
    tweet_id = np.uint64(tweet_id)

    user_data_account, _data_bump = PublicKey.find_program_address([user_id.tobytes()], PublicKey(program_key))

    user_hashtag_account, _hashtag_bump = PublicKey.find_program_address([bytes(hashtag, encoding="utf8"), tweet_id.tobytes(), user_id.tobytes()], PublicKey(program_key))

    print("seed 1", list(bytes(hashtag, encoding="utf8")))
    print("seed 2", list(tweet_id.tobytes()))
    print("seed 3", list(user_id.tobytes()))
    print("account", user_hashtag_account)

    user_account = PublicKey(user_account_key)
    user_token_account = spl_token_instructions.get_associated_token_address(user_account, MINT_KEY)

    program_derived_account, _pda_bump = PublicKey.find_program_address([bytes("token_account", encoding="utf-8")], PublicKey(program_key))
    program_token_account = spl_token_instructions.get_associated_token_address(program_derived_account, MINT_KEY)

    idx = Twitter_Instructions.build(Twitter_Instructions.enum.SendTokens(amount = 1, tweet_id = tweet_id, hashtag = hashtag))

    instruction = TransactionInstruction(
        program_id = program_key,
        data = idx,
        keys = [
            AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),

            AccountMeta(pubkey=PublicKey(user_account_key), is_signer=False, is_writable=False),
            AccountMeta(pubkey=user_id_map_account, is_signer=False, is_writable=False),
            AccountMeta(pubkey=user_data_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_hashtag_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=user_token_account, is_signer=False, is_writable=True),

            AccountMeta(pubkey=program_derived_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=program_token_account, is_signer=False, is_writable=True),

            AccountMeta(pubkey=MINT_KEY, is_signer=False, is_writable=False),

            AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(pubkey=sp.SYS_PROGRAM_ID, is_signer=False, is_writable=False)
        ]
    )

    return instruction


def send_transaction(dev_client, instructions):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    blockhash = dev_client.get_recent_blockhash()['result']['value']['blockhash']
    txn = Transaction(recent_blockhash=blockhash, fee_payer=wallet.public_key)

    for idx in instructions:
        txn.add(idx)

    txn.sign(wallet)

    response = dev_client.send_transaction(
        txn,
        wallet,
        opts=TxOpts(skip_preflight=True, skip_confirmation=True)
    )

    print(response)