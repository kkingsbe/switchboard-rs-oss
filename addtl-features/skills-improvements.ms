Skill searching should be done via the following api, rather than using the skills.sh cli:
curl --request GET \
  --url 'https://skills.sh/api/search?q=<search_query>&limit=50' \
  --header 'Content-Type: application/json' \
  --header 'User-Agent: insomnia/12.3.1' \
  --cookie region=us

