# =========================
# 🔧 Build Stage
# =========================
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build
WORKDIR /src

# ⚡ copy csproj trước để cache restore
COPY apex-backend.csproj ./
RUN dotnet restore apex-backend.csproj

# 📦 copy source
COPY . .

# ⚡ publish optimized
RUN dotnet publish apex-backend.csproj \
    -c Release \
    -o /app/publish \
    /p:UseAppHost=false

# =========================
# 🚀 Runtime Stage
# =========================
FROM mcr.microsoft.com/dotnet/aspnet:8.0 AS runtime
WORKDIR /app

# 🔐 chạy non-root (best practice)
RUN adduser --disabled-password --gecos "" appuser
USER appuser

# 📦 copy build output
COPY --from=build /app/publish .

# 🌐 config
ENV ASPNETCORE_URLS=http://+:5000 \
    ASPNETCORE_ENVIRONMENT=Production \
    DOTNET_RUNNING_IN_CONTAINER=true \
    DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=true \
    ConnectionStrings__LocalDb="Data Source=guardian.db"

EXPOSE 5000

ENTRYPOINT ["dotnet", "apex-backend.dll"]
