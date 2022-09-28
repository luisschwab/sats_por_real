import os
import dotenv
import tweepy
import requests

dotenv.load_dotenv('.env')

client = tweepy.Client(
    consumer_key=os.getenv("CONSUMER_KEY"),
    consumer_secret=os.getenv("CONSUMER_SECRET"),
    access_token=os.getenv("ACCESS_TOKEN"),
    access_token_secret=os.getenv("ACCESS_TOKEN_SECRET")
)


try:
    #binance
    url = os.getenv("API_BINANCE")
    response = requests.get(url, timeout=5); response.raise_for_status()
    data = response.json()
    BTCBRL = float(data['price'])

except requests.exceptions.HTTPError:
    #mercado bitcoin
    url = os.getenv("API_MERCADOBITCOIN")
    response = requests.get(url, timeout=5)
    data = response.json()
    BTCBRL = float(data['ticker']['last'])

BRLSAT = round(10**8/float(BTCBRL))


try:
    url = 'https://bitcoinexplorer.org/api/blocks/tip/height/'
    response = requests.get(url, timeout=5)
    block = response.json()

except requests.exceptions.HTTPError:
    exit()


print(f'丰{BRLSAT}, block {block}')

tweet = '丰' + str(BRLSAT) + '\n' + 'block ' + str(block)
push = client.create_tweet(text=tweet)