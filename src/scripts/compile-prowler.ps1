Remove-Item publish/reaper* -Recurse -Force
git clone https://github.com/CBPS-BPO/Harrier-Common
git -C .\Harrier-Common\ switch development
git -C .\Harrier-Common\ pull
# 
git clone https://github.com/CBPS-BPO/Harrier-Prowler
git -C .\Harrier-Prowler\ switch development
git -C .\Harrier-Prowler\ pull
dotnet publish .\Harrier-Prowler\src\Harrier.Prowler.Service\ --self-contained -o ./publish/prowler -r linux-x64
Compress-Archive .\publish\prowler\* .\publish\prowler.zip