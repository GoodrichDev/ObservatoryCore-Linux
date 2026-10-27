using System;
using System.Text.Json.Serialization;

// Independent installed-source settings constructor capture. No plugin DLL is loaded.
 using System.Collections.Generic; using System.Text.Json;
public class BaseThemeSettings { public int ThemeIndex{get;set;} public int ThemeMode{get;set;} public List<string> ThemeFullName{get;set;}=new(){"Original"}; public List<string> ThemeFileName{get;set;}=new(){"theme.json"}; }




public class NotificationSettings
{
	public NotificationOptions PersonalCodex { get; set; }

	public NotificationOptions Marked { get; set; }

	public NotificationOptions HighValue { get; set; }

	public NotificationOptions Distance { get; set; }

	public NotificationOptions OnScanDetails { get; set; }

	public NotificationOptions NewGalactic { get; set; }

	public NotificationOptions Samples { get; set; }

	public NotificationOptions AtmosphericBody { get; set; }

	public NotificationOptions AtmosphericBodyWithBios { get; set; }

	public NotificationOptions NonAtmosphericBodyWithBios { get; set; }

	public NotificationOptions RadiantStarUpdates { get; set; }

	public NotificationSettings()
	{
		PersonalCodex = new NotificationOptions();
		Marked = new NotificationOptions();
		HighValue = new NotificationOptions();
		Distance = new NotificationOptions();
		OnScanDetails = new NotificationOptions();
		NewGalactic = new NotificationOptions();
		Samples = new NotificationOptions();
		AtmosphericBody = new NotificationOptions();
		AtmosphericBodyWithBios = new NotificationOptions();
		NonAtmosphericBodyWithBios = new NotificationOptions();
		RadiantStarUpdates = new NotificationOptions();
	}
}




public class NotificationOptions
{
	public bool Popup { get; set; }

	public bool Voice { get; set; }

	public bool Plugin { get; set; }

	[JsonIgnore]
	public bool Any => Popup | Voice | Plugin;

	[JsonIgnore]
	public NotificationRendering Rendering => (NotificationRendering)((Popup ? 1 : 0) | (Voice ? 2 : 0) | (Plugin ? 4 : 0));

	public void Or(NotificationOptions other)
	{
		Popup |= other.Popup;
		while (true)
		{
			int num = -666481937;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -310372778)) % 4)
				{
				case 3u:
					break;
				default:
					return;
				case 1u:
					Voice |= other.Voice;
					num = (int)(num2 * 1221557239) ^ -829798209;
					continue;
				case 2u:
					Plugin |= other.Plugin;
					num = ((int)num2 * -1276806946) ^ 0x5ED9DCD6;
					continue;
				case 0u:
					return;
				}
				break;
			}
		}
	}

	public NotificationOptions()
	{
		Popup = true;
		Voice = true;
		Plugin = false;
	}
}

public class BioInsightsSettings {
public int ZoomLevel{get;set;}
public BaseThemeSettings ThemeSettings {get;set;}
public bool PersonalCodexPossibleAny {get;set;}
public bool PersonalCodexPossibleCurrent {get;set;}
public NotificationSettings Notifications {get;set;}
public bool HecateUseDB {get;set;}
public bool HecateUseSpansh {get;set;}
public string NativeOutput {get;set;}
public bool HideNotPresent {get;set;}
public bool ShowBodyFields {get;set;}
public bool ExpandOutput {get;set;}
public bool HideAutoscans {get;set;}
public bool HideAutoscansNoPredictions {get;set;}
public bool ShowHeaders {get;set;}
public bool ShowTotal {get;set;}
public bool ShowSplitters {get;set;}
public int MaximumSystemsReadAll {get;set;}
public string ButtonSize {get;set;}
public bool PopoutAlwaysShow {get;set;}
public bool PopoutShowNoFocus {get;set;}
public bool PopoutShowLeftPanel {get;set;}
public bool PopoutShowRightPanel {get;set;}
public bool PopoutShowDSSMode {get;set;}
public bool PopoutShowRolePanel {get;set;}
public bool PopoutShowSysMap {get;set;}
public bool PopoutShowFSSMode {get;set;}
public int PopoutOpacity {get;set;}
public bool PopoutHideUnlessSupercruiseOrSrv {get;set;}
public bool PopoutHidePopulated {get;set;}
public bool StellarOverlaySendFSSNotification {get;set;}
public bool StellarOverlaySendDSSNotification {get;set;}
public bool StellarOverlayUseRadar {get;set;}
public int FSSNotificationFontSize {get;set;}
public int FSSNotificationOpacity {get;set;}
public int FSSNotificationXPos {get;set;}
public int FSSNotificationYPos {get;set;}
public int DSSNotificationXPos {get;set;}
public int DSSNotificationYPos {get;set;}
public int FSSNotificationBorderWidth {get;set;}
public long FSSNotificationTTL {get;set;}
public bool UseCustomNotificationColours {get;set;}
public string TitleColorHex {get;set;}
public string BackgroundColorHex {get;set;}
public string BorderColorHex {get;set;}
public string ColorHex {get;set;}
public double MinBodyValueMillions {get;set;}
public double MinIndividualBiologicalValueMillions {get;set;}
public string DataDirectory {get;set;}
public bool InstallUpdates {get;set;}
public bool InstallBetas {get;set;}
public bool UseCannonData {get;set;}
public bool SaveSystemData {get;set;}
public string DebugBody {get;set;}
public bool VerboseLogging {get;set;}
public bool SendDebugData {get;set;}
public BioInsightsSettings()
	{
		while (true)
		{
			int num = 1498080020;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x72FBAAEF)) % 36)
				{
				case 5u:
					break;
				case 12u:
					ButtonSize = "  Normal";
					num = 322158454;
					continue;
				case 4u:
					ThemeSettings = new BaseThemeSettings();
					ThemeSettings.ThemeFullName = new List<string>
					{
						"Original",
						"Elite Orange"
					};
					num = 1176365090;
					continue;
				case 13u:
					HideNotPresent = false;
					MaximumSystemsReadAll = 100;
					num = (int)((num2 * 2132100332) ^ 0x4976CAF6);
					continue;
				case 19u:
					InstallBetas = false;
					ExpandOutput = true;
					HideAutoscans = false;
					num = ((int)num2 * -2123887058) ^ 0x73ADD767;
					continue;
				case 26u:
					PopoutShowFSSMode = true;
					num = (int)((num2 * 1125577563) ^ 0x673C16A7);
					continue;
				case 22u:
					InstallUpdates = true;
					num = (int)((num2 * 371306592) ^ 0x4755929C);
					continue;
				case 24u:
					PopoutShowNoFocus = true;
					PopoutShowLeftPanel = true;
					num = ((int)num2 * -324109041) ^ 0x46515AA;
					continue;
				case 29u:
					PopoutShowRightPanel = true;
					num = ((int)num2 * -547446894) ^ 0x1EA32081;
					continue;
				case 21u:
					PopoutShowSysMap = true;
					num = (int)(num2 * 352104993) ^ -1986733640;
					continue;
				case 31u:
					ShowSplitters = false;
					num = (int)(num2 * 1707702071) ^ -1884534574;
					continue;
				case 16u:
					ColorHex = "FFFF6300";
					BorderColorHex = "FFFF3D3A";
					num = 27702666;
					continue;
				case 27u:
					FSSNotificationOpacity = 80;
					num = ((int)num2 * -875024517) ^ -1644716556;
					continue;
				case 30u:
					PopoutOpacity = 100;
					num = ((int)num2 * -550273709) ^ 0x9F594AE;
					continue;
				case 18u:
					HecateUseSpansh = true;
					num = (int)((num2 * 1719564935) ^ 0x3425D442);
					continue;
				case 8u:
					PopoutShowDSSMode = true;
					PopoutShowRolePanel = true;
					num = ((int)num2 * -412880939) ^ -1220680782;
					continue;
				case 20u:
					FSSNotificationXPos = 2;
					FSSNotificationYPos = 2;
					num = (int)(num2 * 594176487) ^ -936641285;
					continue;
				case 11u:
					FSSNotificationFontSize = 12;
					num = ((int)num2 * -901091668) ^ -2013029241;
					continue;
				case 17u:
					ZoomLevel = 100;
					num = (int)(num2 * 697306944) ^ -2045381981;
					continue;
				case 34u:
					FSSNotificationTTL = 6000L;
					UseCustomNotificationColours = false;
					num = (int)(num2 * 66019808) ^ -433061987;
					continue;
				case 33u:
					SaveSystemData = true;
					num = ((int)num2 * -882376144) ^ -1279955763;
					continue;
				case 3u:
					HecateUseDB = true;
					num = ((int)num2 * -1165721437) ^ -1660005504;
					continue;
				case 6u:
					TitleColorHex = "FFFF7B3A";
					num = 143283695;
					continue;
				case 0u:
					DSSNotificationYPos = 98;
					num = (int)((num2 * 1305734648) ^ 0x4F8C37F4);
					continue;
				case 23u:
					PersonalCodexPossibleAny = true;
					PersonalCodexPossibleCurrent = false;
					VerboseLogging = false;
					MinIndividualBiologicalValueMillions = 12.0;
					num = (int)((num2 * 1855682286) ^ 0x581F3BC3);
					continue;
				case 7u:
					DebugBody = "";
					SendDebugData = true;
					num = ((int)num2 * -1972197389) ^ 0x1DD7BE4D;
					continue;
				case 1u:
					BackgroundColorHex = "B0000000";
					ShowHeaders = true;
					ShowTotal = false;
					num = 1270446252;
					continue;
				case 32u:
					DSSNotificationXPos = 2;
					num = (int)(num2 * 949714608) ^ -1850943001;
					continue;
				case 15u:
					StellarOverlaySendFSSNotification = true;
					StellarOverlaySendDSSNotification = true;
					num = ((int)num2 * -571549608) ^ 0x5540C5E2;
					continue;
				case 9u:
					StellarOverlayUseRadar = true;
					num = ((int)num2 * -964584797) ^ 0x72B2F523;
					continue;
				case 2u:
					HideAutoscansNoPredictions = true;
					PopoutAlwaysShow = true;
					num = ((int)num2 * -1249054011) ^ -1538539195;
					continue;
				case 25u:
					ThemeSettings.ThemeFileName = new List<string>
					{
						"theme.json",
						"theme-orange.json"
					};
					ThemeSettings.ThemeIndex = 1;
					num = 2033029436;
					continue;
				case 10u:
					MinBodyValueMillions = 20.0;
					num = ((int)num2 * -907862532) ^ 0x76F1C746;
					continue;
				case 14u:
					FSSNotificationBorderWidth = 2;
					num = ((int)num2 * -1788361062) ^ -142041047;
					continue;
				case 28u:
					Notifications = new NotificationSettings();
					num = (int)((num2 * 1807794687) ^ 0x350DFFD7);
					continue;
				default:
					NativeOutput = " Enabled";
					return;
				}
				break;
			}
		}
	}
}
public class Program { public static void Main() {Console.WriteLine(JsonSerializer.Serialize(new BioInsightsSettings(),new JsonSerializerOptions{WriteIndented=true}));}}

[Flags] public enum NotificationRendering { Popup=1, Voice=2, Plugin=4 }
