{
    "success": false,
    "properties": null,
    "error": "GA4 API error: 403 Forbidden - {\n  \"error\": {\n    \"code\": 403,\n    \"message\": \"Google Analytics Admin API has not been used in project 313968973975 before or it is disabled.
  Enable it by visiting https://console.developers.google.com/apis/api/analyticsadmin.googleapis.com/overview?project=313968973975 then retry. If you enabled this API recently, wait a few minutes for
  the action to propagate to our systems and retry.\",\n    \"status\": \"PERMISSION_DENIED\",\n    \"details\": [\n      {\n        \"@type\": \"type.googleapis.com/google.rpc.ErrorInfo\",\n
  \"reason\": \"SERVICE_DISABLED\",\n        \"domain\": \"googleapis.com\",\n        \"metadata\": {\n          \"service\": \"analyticsadmin.googleapis.com\",\n          \"consumer\":
  \"projects/313968973975\",\n          \"activationUrl\": \"https://console.developers.google.com/apis/api/analyticsadmin.googleapis.com/overview?project=313968973975\",\n          \"serviceTitle\":
  \"Google Analytics Admin API\",\n          \"containerInfo\": \"313968973975\"\n        }\n      },\n      {\n        \"@type\": \"type.googleapis.com/google.rpc.LocalizedMessage\",\n
  \"locale\": \"en-US\",\n        \"message\": \"Google Analytics Admin API has not been used in project 313968973975 before or it is disabled. Enable it by visiting
  https://console.developers.google.com/apis/api/analyticsadmin.googleapis.com/overview?project=313968973975 then retry. If you enabled this API recently, wait a few minutes for the action to
  propagate to our systems and retry.\"\n      },\n      {\n        \"@type\": \"type.googleapis.com/google.rpc.Help\",\n        \"links\": [\n          {\n            \"description\": \"Google
  developers console API activation\",\n            \"url\": \"https://console.developers.google.com/apis/api/analyticsadmin.googleapis.com/overview?project=313968973975\"\n          }\n        ]\n
     }\n    ]\n  }\n}\n"
  }

⏺ You need to enable the Google Analytics Admin API in your Google Cloud project.

  1. Visit: https://console.developers.google.com/apis/api/analyticsadmin.googleapis.com/overview?project=313968973975
  2. Click "Enable"
  3. Wait 1-2 minutes for it to propagate
  4. Try again:
  curl http://localhost:3000/connectors/ga4/properties

  You'll also want to enable the Google Analytics Data API (for pulling actual analytics data):
  https://console.developers.google.com/apis/api/analyticsdata.googleapis.com/overview?project=313968973975
  
  
  
  
  
  
  
  
    The issue is not the property ID. The error says the Google Analytics Data API needs to be enabled in your Google Cloud project (ID: 313968973975), not in the Analytics property.
  
    These are different:
    - GA4 Property (256747091) - your Analytics data
    - Google Cloud Project (313968973975) - where your OAuth credentials live
  
    Go to this exact URL and click "Enable":
    https://console.developers.google.com/apis/api/analyticsdata.googleapis.com/overview?project=313968973975
  
    This is a different API from the Admin API you already enabled. You need both:
    1. ✅ Analytics Admin API (for listing properties) - already enabled
    2. ❌ Analytics Data API (for running reports) - needs to be enabled
