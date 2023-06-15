from pkgutil import get_data
from solana.rpc.api import Client
import concurrent.futures as cf
import numpy as np
import time

from rpc_funcs import *
from log import *
from transactions import *
from error_codes import *

# connect to solana node
quick_node_dev = "https://api.devnet.solana.com"

dev_client = Client(quick_node_dev)

if (not dev_client.is_connected()):
    log_error("cannot connect to quicknode endpoint.")
    exit()

current_slot = get_slot(dev_client)

last_signature = None
while(True):

    signatures, last_signature = get_signatures(quick_node_dev, current_slot, np.inf, last_signature)

    if (len(signatures) > 0):
        transactions = get_transactions(quick_node_dev, signatures)


        for transaction in transactions:
            data = get_data_from_transaction(transaction["transaction"])
            
            for d in data:
                args = d["args"]
                if (isinstance(args, Twitter_Instructions.enum.Register)):
                    tweet_id = args.tweet_id
                    user_pubkey = d["user"]

                    tweet_text, user_id, hashtags = process_tweet(tweet_id)

                    if (user_pubkey not in tweet_text):
                        log_error("Mismatch in public keys!")
                        set_user_error(dev_client, user_pubkey, PUBKEY_MISMATCH)
                        continue

                    instructions = []
                    instructions.append(get_set_user_id_idx(user_id, user_pubkey))
                    instructions.append(get_set_error_idx(USER_ID_ACCOUNT_INITED, user_pubkey))

                    send_transaction(dev_client, instructions)

                if (isinstance(args, Twitter_Instructions.enum.CheckHashTag)):
                    print("Have a hashtag!")
                    tweet_id = args.tweet_id
                    hashtag = args.hashtag
                    user_pubkey = d["user"]

                    if (hashtag not in VALID_HASHTAGS):
                        log_error("hashtag " + args.hashtag + " not in valid list!")
                        set_user_error(dev_client, user_pubkey, INVALID_HASHTAG)
                        continue

                    tweet_text, twitter_id_from_tweet, hashtags = process_tweet(tweet_id)
                    print("tweet_text: ", tweet_text)
                    print("twitter_id_from_tweet: ", twitter_id_from_tweet)
                    print("hashtags: ", hashtags)

                    if (hashtag not in hashtags):
                        log_error("hashtag " + args.hashtag + " not present in tweet!")
                        set_user_error(dev_client, user_pubkey, HASHTAG_MISMATCH)
                        continue

                    twitter_id_from_map, error_code_from_map = get_user_id_map_data(dev_client, user_pubkey)

                    if (twitter_id_from_map != twitter_id_from_tweet):
                        log_error("twitter ids don't match: " + str(twitter_id_from_map) + " "  + str(twitter_id_from_tweet))
                        set_user_error(dev_client, user_pubkey, TWITTER_ID_MISMATCH)
                        continue

                    instructions = []
                    instructions.append(get_send_hashtag_reward_idx(user_pubkey, twitter_id_from_map, tweet_id, hashtag))
                    instructions.append(get_set_error_idx(NO_ERROR, user_pubkey))

                    send_transaction(dev_client, instructions)

                    print(args)

                if (isinstance(args, Twitter_Instructions.enum.CheckRetweet)):
                    print("Have a Retweet!")
                    tweet_id = args.tweet_id
                    hashtag = args.hashtag
                    user_pubkey = d["user"]

                    tweet_text, twitter_id_from_tweet, hashtags = process_tweet(tweet_id)
                    
                    # check if tweet author was daoplays
                    daoplays_twitter_id = 1532485814051012608
                    if (twitter_id_from_tweet != daoplays_twitter_id):
                        print ("retweet wasn't orignally a daoplays!")
                        continue

                    retweeters = get_retweeters(tweet_id)

                    twitter_id_from_map, error_code_from_map = get_user_id_map_data(dev_client, user_pubkey)

                    if twitter_id_from_map not in retweeters:
                        print ("user not found in retweet list")
                        continue

                    mark = get_reward_mark_data(dev_client, hashtag, tweet_id, twitter_id_from_map)

                    if (mark):
                        log_error("tweet " + str(tweet_id) + " has already been rewteeted")
                        set_user_error(dev_client, user_pubkey, ALREADY_RETWEETED)
                        continue


                    
                    instructions = []
                    instructions.append(get_send_hashtag_reward_idx(user_pubkey, twitter_id_from_map, tweet_id, "retweet"))
                    instructions.append(get_set_error_idx(NO_ERROR, user_pubkey))

                    send_transaction(dev_client, instructions)

                if (isinstance(args, Twitter_Instructions.enum.CheckFollower)):
                    print("Have a Follower!")
                    user_pubkey = d["user"]

                    twitter_id, error_code = get_user_id_map_data(dev_client, user_pubkey)
                    following =  check_if_user_is_following(twitter_id)

                    if not following :
                        log_error("user not following")
                        set_user_error(dev_client, user_pubkey, NOT_FOLLOWING)
                        continue
                    
                    instructions = []
                    instructions.append(get_new_follower_idx(user_id, user_pubkey))
                    instructions.append(get_set_error_idx(NO_ERROR, user_pubkey))

                    send_transaction(dev_client, instructions)
        
    time.sleep(10)