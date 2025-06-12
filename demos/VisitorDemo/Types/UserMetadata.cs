namespace VisitorDemo;

public class UserMetadata(int version) : IType<User>, IStructType<User>
{
    public void Accept<V>(V visitor) where V : ITypeVisitor<User>, allows ref struct => visitor.VisitStruct(Getters.Identity, this);

    void IStructType<User>.Accept<V>(V visitor)
    {
        visitor.VisitField("Name", (ref User s) => ref s.Name, StringMetadata.Instance);
        if (version >= 2)
            visitor.VisitField("Age", (ref User s) => ref s.Age, IntMetadata.Instance);
        visitor.VisitField("Contacts", (ref User s) => ref s.Contacts, new VersionedContactsMetadata(version));
    }
}
