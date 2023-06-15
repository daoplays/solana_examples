# error codes start from 0
NO_ERROR = 0  # everything is fine
PUBKEY_MISMATCH = 1 # when registering the public key in the tweet doesn't match the public key of the wallet registering
HASHTAG_MISMATCH = 2 # when sending a tweet with a hashtag, if the hashtags don't contain the expected one
TWITTER_ID_MISMATCH = 3 # when the user id of the tweet with a hashtag doesn't match the user id of the account
INVALID_HASHTAG = 4 # user has tried to submit an invalid hashtag
NOT_FOLLOWING = 5 # the user asked for a follow reward but isn't following
ALREADY_RETWEETED = 6 # the user tried to request a reward for a tweet already retweeted

# info codes start from 100
USER_ID_ACCOUNT_CREATED = 100 # we have set up the user id account
USER_ID_ACCOUNT_INITED = 101 # we have inited the user id account