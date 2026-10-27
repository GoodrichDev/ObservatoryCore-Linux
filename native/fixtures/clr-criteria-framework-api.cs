// Metadata-only public API inventory. Does not construct any Framework classes.
using System.Reflection;
using System.Text.Json;

var assembly = Assembly.LoadFrom(Path.GetFullPath(args[0]));
const BindingFlags flags = BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly;
string Shape(Type type)
{
    var lines = new List<string> { "TYPE " + type.FullName + " : " + type.BaseType + " " + type.Attributes };
    lines.AddRange(type.GetCustomAttributesData().Where(RelevantAttribute).Select(a => "TYPE-ATTRIBUTE " + a));
    lines.AddRange(type.GetInterfaces().Select(i => "INTERFACE " + i));
    foreach (var member in type.GetMembers(flags))
    {
        string signature = member.MemberType + " " + member;
        if (member is FieldInfo { IsLiteral: true } field)
            signature += " = " + field.GetRawConstantValue();
        if (member is PropertyInfo property)
            signature += " get=" + property.GetMethod?.IsPublic + " set=" + property.SetMethod?.IsPublic;
        if (member is MethodBase method)
            signature += " params=" + string.Join(",", method.GetParameters().Select(p => p.Name + ":" + p.Attributes + (p.HasDefaultValue ? "=" + (p.RawDefaultValue ?? "null") : "")));
        lines.Add(signature);
        lines.AddRange(member.GetCustomAttributesData().Where(RelevantAttribute).Select(a => "MEMBER-ATTRIBUTE " + member + " " + a));
    }
    return string.Join("\n", lines.OrderBy(x => x, StringComparer.Ordinal));
}
bool RelevantAttribute(CustomAttributeData attribute) =>
    !attribute.AttributeType.FullName!.StartsWith("System.Runtime.CompilerServices.", StringComparison.Ordinal);
Console.WriteLine(JsonSerializer.Serialize(new {
    identity = assembly.FullName,
    types = assembly.GetExportedTypes().OrderBy(t => t.FullName, StringComparer.Ordinal)
        .ToDictionary(t => t.FullName!, t => Shape(t))
}));
