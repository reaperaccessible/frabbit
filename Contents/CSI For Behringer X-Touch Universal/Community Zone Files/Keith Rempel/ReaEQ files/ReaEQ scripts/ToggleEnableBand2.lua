-- Toggles a given ReaEQ band between enabled and disabled.
-- Change the value of band number here.
BandNumber = 2

tr = reaper.GetSelectedTrack(0, 0)
EqIndex = reaper.TrackFX_GetEQ(tr, 0)
BandNum = BandNumber-1

retval, Enabled = reaper.TrackFX_GetNamedConfigParm(tr,EqIndex, 'BANDENABLED'..BandNum)

if retval == false then
    reaper.osara_outputMessage ("No EQ in focus")
    return
end

Enabled = tonumber(Enabled)

if Enabled == 1 then
    reaper.TrackFX_SetNamedConfigParm( tr, EqIndex, 'BANDENABLED'..BandNum, 0 )
    Message = 'Disabled band '..BandNumber
else
    reaper.TrackFX_SetNamedConfigParm( tr, EqIndex, 'BANDENABLED'..BandNum, 1 )
    Message = 'Enabled band '..BandNumber
end

reaper.osara_outputMessage (Message)