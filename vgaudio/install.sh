wget https://ci.appveyor.com/api/buildjobs/wa2ie68wd0eq51sw/artifacts/VGAudioCli.zip
unzip VGAudioCli.zip
mv netcoreapp2.0/* .
rm -r net451 net451_standalone netcoreapp2.0 VGAudioCli.zip
