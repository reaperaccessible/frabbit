--Assign the number of the band you would like to control to BandNumber below.
BandNumber = 2

-- If you do not wish to include band type in the cycle, change it's value to 0.
LowShelf = 1
HighShelf = 1
Band = 1
LowPass = 1
HighPass = 1
AllPass = 0
Notch = 0
BandPass = 1
BandAlt = 0
BandAlt2 = 0
ParallelBandPass = 0

BandTypeIndex = 'BANDTYPE'..BandNumber-1


function SetBand(NewBand, BandNum)
    if _G[NewBand] == 1 then
        reaper.TrackFX_SetNamedConfigParm(tr, EqIndex, BandTypeIndex, BandNum)
        reaper.osara_outputMessage (NewBand)
        return
    else OldBand = BandNum
        return
    end
end

tr = reaper.GetSelectedTrack(0, 0)
EqIndex = reaper.TrackFX_GetEQ(tr, 0)

retval, OldBand = reaper.TrackFX_GetNamedConfigParm(tr,EqIndex, BandTypeIndex)

if retval == false then
    reaper.osara_outputMessage ("No EQ found")
    return
end

OldBand = OldBand*1

if OldBand == 0 then
    SetBand('HighShelf', 1)
end
if OldBand == 1 then
    SetBand('Band', 8)
end
if OldBand == 8 then
    SetBand('LowPass', 3)
end
if OldBand == 3 then
    SetBand('HighPass', 4)
end
if OldBand == 4 then
    SetBand('AllPass', 5)
end
if OldBand == 5 then
    SetBand('Notch', 6)
end
if OldBand == 6 then
    SetBand('BandPass', 7)
end
if OldBand == 7 then
    SetBand('ParallelBandPass', 10)
end
if OldBand == 10 then
    SetBand('BandAlt', 2)
end
if OldBand == 2 then
    SetBand('BandAlt2', 9)
end
if OldBand == 9 then
    SetBand('LowShelf', 0)
end

