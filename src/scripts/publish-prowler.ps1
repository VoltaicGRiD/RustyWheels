$resourceGroupName = "har-services-dev"
az webapp deployment source config-zip --src .\publish\prowler.zip -n "har-prowler-dev" -g $resourceGroupName