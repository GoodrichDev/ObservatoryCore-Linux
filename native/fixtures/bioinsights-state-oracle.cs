// Verbatim installed makeKey and ReadFileVersion100 method bodies; the DLL is never loaded.
using System;using System.IO;using System.Collections.Generic;using System.Text.Json;


public enum GenusType
{
	Unknown = 0,
	Codex_Ent_Aleoids_Genus_Name = 1,
	Codex_Ent_Bacterial_Genus_Name = 2,
	Codex_Ent_Cactoid_Genus_Name = 3,
	Codex_Ent_Clypeus_Genus_Name = 4,
	Codex_Ent_Clepeus_Genus_Name = Codex_Ent_Clypeus_Genus_Name,
	Codex_Ent_Conchas_Genus_Name = 5,
	Codex_Ent_Electricae_Genus_Name = 6,
	Codex_Ent_Fonticulus_Genus_Name = 7,
	Codex_Ent_Shrubs_Genus_Name = 8,
	Codex_Ent_Fumerolas_Genus_Name = 9,
	Codex_Ent_Fungoids_Genus_Name = 10,
	Codex_Ent_Osseus_Genus_Name = 11,
	Codex_Ent_Recepta_Genus_Name = 12,
	Codex_Ent_Stratum_Genus_Name = 13,
	Codex_Ent_Tubus_Genus_Name = 14,
	Codex_Ent_Tussocks_Genus_Name = 15,
	Codex_Ent_Brancae_Name = 16,
	Codex_Ent_Seed_Name = Codex_Ent_Brancae_Name,
	Codex_Ent_Ground_Struct_Ice_Name = 17,
	Codex_Ent_Vents_Name = 18,
	Codex_Ent_Cone_Name = 19,
	Codex_Ent_Sphere_Name = 20,
	Codex_Ent_Tube_Name = 21
}



public enum SpeciesType
{
	Unknown,
	Codex_Ent_Aleoids_01_Name,
	Codex_Ent_Aleoids_02_Name,
	Codex_Ent_Aleoids_03_Name,
	Codex_Ent_Aleoids_04_Name,
	Codex_Ent_Aleoids_05_Name,
	Codex_Ent_Bacterial_01_Name,
	Codex_Ent_Bacterial_02_Name,
	Codex_Ent_Bacterial_03_Name,
	Codex_Ent_Bacterial_04_Name,
	Codex_Ent_Bacterial_05_Name,
	Codex_Ent_Bacterial_06_Name,
	Codex_Ent_Bacterial_07_Name,
	Codex_Ent_Bacterial_08_Name,
	Codex_Ent_Bacterial_09_Name,
	Codex_Ent_Bacterial_10_Name,
	Codex_Ent_Bacterial_11_Name,
	Codex_Ent_Bacterial_12_Name,
	Codex_Ent_Bacterial_13_Name,
	Codex_Ent_Cactoid_01_Name,
	Codex_Ent_Cactoid_02_Name,
	Codex_Ent_Cactoid_03_Name,
	Codex_Ent_Cactoid_04_Name,
	Codex_Ent_Cactoid_05_Name,
	Codex_Ent_Clypeus_01_Name,
	Codex_Ent_Clypeus_02_Name,
	Codex_Ent_Clypeus_03_Name,
	Codex_Ent_Conchas_01_Name,
	Codex_Ent_Conchas_02_Name,
	Codex_Ent_Conchas_03_Name,
	Codex_Ent_Conchas_04_Name,
	Codex_Ent_Electricae_01_Name,
	Codex_Ent_Electricae_02_Name,
	Codex_Ent_Fonticulus_01_Name,
	Codex_Ent_Fonticulus_02_Name,
	Codex_Ent_Fonticulus_03_Name,
	Codex_Ent_Fonticulus_04_Name,
	Codex_Ent_Fonticulus_05_Name,
	Codex_Ent_Fonticulus_06_Name,
	Codex_Ent_Fumerolas_01_Name,
	Codex_Ent_Fumerolas_02_Name,
	Codex_Ent_Fumerolas_03_Name,
	Codex_Ent_Fumerolas_04_Name,
	Codex_Ent_Fungoids_01_Name,
	Codex_Ent_Fungoids_02_Name,
	Codex_Ent_Fungoids_03_Name,
	Codex_Ent_Fungoids_04_Name,
	Codex_Ent_Osseus_01_Name,
	Codex_Ent_Osseus_02_Name,
	Codex_Ent_Osseus_03_Name,
	Codex_Ent_Osseus_04_Name,
	Codex_Ent_Osseus_05_Name,
	Codex_Ent_Osseus_06_Name,
	Codex_Ent_Recepta_01_Name,
	Codex_Ent_Recepta_02_Name,
	Codex_Ent_Recepta_03_Name,
	Codex_Ent_Shrubs_01_Name,
	Codex_Ent_Shrubs_02_Name,
	Codex_Ent_Shrubs_03_Name,
	Codex_Ent_Shrubs_04_Name,
	Codex_Ent_Shrubs_05_Name,
	Codex_Ent_Shrubs_06_Name,
	Codex_Ent_Shrubs_07_Name,
	Codex_Ent_Stratum_01_Name,
	Codex_Ent_Stratum_02_Name,
	Codex_Ent_Stratum_03_Name,
	Codex_Ent_Stratum_04_Name,
	Codex_Ent_Stratum_05_Name,
	Codex_Ent_Stratum_06_Name,
	Codex_Ent_Stratum_07_Name,
	Codex_Ent_Stratum_08_Name,
	Codex_Ent_Tubus_01_Name,
	Codex_Ent_Tubus_02_Name,
	Codex_Ent_Tubus_03_Name,
	Codex_Ent_Tubus_04_Name,
	Codex_Ent_Tubus_05_Name,
	Codex_Ent_Tussocks_01_Name,
	Codex_Ent_Tussocks_02_Name,
	Codex_Ent_Tussocks_03_Name,
	Codex_Ent_Tussocks_04_Name,
	Codex_Ent_Tussocks_05_Name,
	Codex_Ent_Tussocks_06_Name,
	Codex_Ent_Tussocks_07_Name,
	Codex_Ent_Tussocks_08_Name,
	Codex_Ent_Tussocks_09_Name,
	Codex_Ent_Tussocks_10_Name,
	Codex_Ent_Tussocks_11_Name,
	Codex_Ent_Tussocks_12_Name,
	Codex_Ent_Tussocks_13_Name,
	Codex_Ent_Tussocks_14_Name,
	Codex_Ent_Tussocks_15_Name,
	Codex_Ent_Seed_Name,
	Codex_Ent_SeedABCD_01_Name,
	Codex_Ent_SeedABCD_02_Name,
	Codex_Ent_SeedABCD_03_Name,
	Codex_Ent_SeedEFGH_Name,
	Codex_Ent_SeedEFGH_01_Name,
	Codex_Ent_SeedEFGH_03_Name,
	Codex_Ent_SeedEFGH_02_Name,
	Codex_Ent_Ground_Struct_Ice_Name,
	Codex_Ent_Vents_Name,
	Codex_Ent_Cone_Name,
	Codex_Ent_Sphere_Name,
	Codex_Ent_SphereABCD_01_Name,
	Codex_Ent_SphereABCD_02_Name,
	Codex_Ent_SphereABCD_03_Name,
	Codex_Ent_SphereEFGH_Name,
	Codex_Ent_SphereEFGH_01_Name,
	Codex_Ent_SphereEFGH_02_Name,
	Codex_Ent_SphereEFGH_03_Name,
	Codex_Ent_Tube_Name,
	Codex_Ent_TubeEFGH_Name,
	Codex_Ent_TubeEFGH_01_Name,
	Codex_Ent_TubeEFGH_02_Name,
	Codex_Ent_TubeEFGH_03_Name,
	Codex_Ent_TubeABCD_01_Name,
	Codex_Ent_TubeABCD_02_Name,
	Codex_Ent_TubeABCD_03_Name
}



public enum VariantColour
{
	Unknown,
	Amethyst,
	Aquamarine,
	Blue,
	Cobalt,
	Cyan,
	Emerald,
	Gold,
	Green,
	Grey,
	Indigo,
	Lime,
	Magenta,
	Maroon,
	Mauve,
	Mulberry,
	Ocher,
	Orange,
	Peach,
	Red,
	Sage,
	Teal,
	Turquoise,
	White,
	Yellow,
	Horizons
}

public class Reference {
private ulong makeKey(GenusType genus, SpeciesType species, VariantColour variant)
	{
		long num = (long)genus << 32;
		ulong num2 = (ulong)((long)species << 16);
		ulong num3 = (ulong)variant;
		return (ulong)num | num2 | num3;
	}
private Dictionary<ulong, ulong> ReadFileVersion100(BinaryReader br)
	{
		Dictionary<ulong, ulong> dictionary = new Dictionary<ulong, ulong>();
		uint num = _202B_202B_200B_206F_206C_200E_200C_206F_206D_202A_202E_202A_206E_202B_202E_202D_200C_200C_206B_200B_206B_202E_202D_200F_200C_202E_200D_206D_202B_200F_206F_200B_206B_206B_200E_206C_206B_200F_200D_206D_202E(br);
		int num4 = default;
		ulong value = default;
		ulong key = default;
		while (true)
		{
			int num2 = 1628934797;
			while (true)
			{
				uint num3;
				switch ((num3 = (uint)(num2 ^ 0x63BC306F)) % 10)
				{
				case 9u:
					break;
				case 5u:
				{
					int num7;
					if (num4 < num)
					{
						num2 = 880567849;
						num7 = num2;
					}
					else
					{
						num2 = 2122325212;
						num7 = num2;
					}
					continue;
				}
				case 8u:
					num4++;
					num2 = 1397485814;
					continue;
				case 2u:
				{
					value = _200C_202D_202C_200E_206A_202D_200F_202A_200E_206B_206B_206C_200F_202C_206C_200B_200F_200E_206E_206F_206C_200E_206B_206F_200E_202C_202A_200D_200E_206E_202B_206E_200C_200C_206F_206A_206B_206F_200F_206A_202E(br);
					int num5;
					int num6;
					if (dictionary.ContainsKey(key))
					{
						num5 = -546096839;
						num6 = num5;
					}
					else
					{
						num5 = -1755745720;
						num6 = num5;
					}
					num2 = num5 ^ (int)(num3 * 937744210);
					continue;
				}
				case 6u:
					num4 = 0;
					num2 = ((int)num3 * -1248824067) ^ -503371464;
					continue;
				case 0u:
					key = _200C_202D_202C_200E_206A_202D_200F_202A_200E_206B_206B_206C_200F_202C_206C_200B_200F_200E_206E_206F_206C_200E_206B_206F_200E_202C_202A_200D_200E_206E_202B_206E_200C_200C_206F_206A_206B_206F_200F_206A_202E(br);
					num2 = 258501841;
					continue;
				case 3u:
					num2 = (int)(num3 * 1510809648) ^ -746925434;
					continue;
				case 1u:
					dictionary.Add(key, value);
					num2 = 1413561943;
					continue;
				case 4u:
					dictionary[key] = value;
					num2 = ((int)num3 * -805065578) ^ -504337269;
					continue;
				default:
					return dictionary;
				}
				break;
			}
		}
	}
static uint _202B_202B_200B_206F_206C_200E_200C_206F_206D_202A_202E_202A_206E_202B_202E_202D_200C_200C_206B_200B_206B_202E_202D_200F_200C_202E_200D_206D_202B_200F_206F_200B_206B_206B_200E_206C_206B_200F_200D_206D_202E(BinaryReader P_0)
	{
		return P_0.ReadUInt32();
	}
static ulong _200C_202D_202C_200E_206A_202D_200F_202A_200E_206B_206B_206C_200F_202C_206C_200B_200F_200E_206E_206F_206C_200E_206B_206F_200E_202C_202A_200D_200E_206E_202B_206E_200C_200C_206F_206A_206B_206F_200F_206A_202E(BinaryReader P_0)
	{
		return P_0.ReadUInt64();
	}
public static void Main(){
var r=new Reference();var first=r.makeKey(GenusType.Codex_Ent_Bacterial_Genus_Name,SpeciesType.Codex_Ent_Bacterial_01_Name,VariantColour.Green);
var second=r.makeKey(GenusType.Codex_Ent_Aleoids_Genus_Name,SpeciesType.Codex_Ent_Aleoids_01_Name,VariantColour.Red);
using var memory=new MemoryStream();using(var writer=new BinaryWriter(memory,System.Text.Encoding.UTF8,true)){writer.Write(1112100681u);writer.Write(100u);writer.Write(4u);writer.Write(first);writer.Write(1UL<<18);writer.Write(second);writer.Write(1UL<<1);writer.Write(ulong.MaxValue);writer.Write(ulong.MaxValue);writer.Write(first);writer.Write((1UL<<18)|(1UL<<42));}
memory.Position=8;using var reader=new BinaryReader(memory);var decoded=r.ReadFileVersion100(reader);var entries=new List<object>();foreach(var pair in decoded)entries.Add(new{Key=pair.Key,Regions=pair.Value});
Console.WriteLine(JsonSerializer.Serialize(new{Format="BiologicalCodexEntry v100",BytesBase64=Convert.ToBase64String(memory.ToArray()),Entries=entries},new JsonSerializerOptions{WriteIndented=true}));
}}
