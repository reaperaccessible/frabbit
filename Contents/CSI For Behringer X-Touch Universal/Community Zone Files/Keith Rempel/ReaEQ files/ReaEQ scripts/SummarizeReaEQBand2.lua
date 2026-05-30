BandNumber = 2
band = (BandNumber-1)*3

tr = reaper.GetSelectedTrack(0, 0)
EqIndex = reaper.TrackFX_GetEQ(tr, 0)

retval, gain = reaper.TrackFX_GetFormattedParamValue(tr, EqIndex, band+1)
retval, freq = reaper.TrackFX_GetFormattedParamValue(tr, EqIndex, band)
retval, type = reaper.TrackFX_GetNamedConfigParm(tr,EqIndex, 'BANDTYPE'..BandNumber-1)
retval, enabled = reaper.TrackFX_GetNamedConfigParm(tr,EqIndex, 'BANDENABLED'..BandNumber-1)

-- Exit if no ReaEQ
if retval == false then
  reaper.osara_outputMessage ("No EQ found")
  return
end


type = type+0
gain = tonumber(gain)
freq = math.floor(tonumber(freq))
enabled = tonumber(enabled)
-- convert band type to text
if type == 0 then
  TypeText = "low shelf"
elseif type == 1 then
  TypeText = "high shelf"
elseif type == 2 then
  TypeText = "band alt 2"
elseif type == 3 then
  TypeText = "low pass"
elseif type == 4 then
  TypeText = "high pass"
elseif type == 5 then
  TypeText = "all pass"
elseif type == 6 then
  TypeText = "notch"
elseif type == 7 then
  TypeText = "band pass"
elseif type == 8 then
  TypeText = "band"
elseif type == 9 then
  TypeText = "band alt 1"
elseif type == 10 then
  TypeText = "parallel band pass"
end

-- convert gain to text
if gain > 0 then
  GainText = gain..'DB boost'
elseif gain < 0 then
  GainText = math.abs(gain)..' DB cut'
else GainText = ' '
end

-- convert Frequency to text
if freq < 1000 then
  FreqText = freq..' hertz'
else
  freq = math.floor(freq/100)/10
  FreqText = freq..' K'
end

if enabled == 1 then
  message = TypeText..' '..GainText..' at '..FreqText
else
  message = 'Band '..BandNumber..' disabled'
end



  reaper.osara_outputMessage (message)